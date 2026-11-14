//! Solution to exercise 6: stop work cleanly with `select!` and a `CancellationToken`.
//!
//! Implement two functions:
//!
//! - `worker`: take jobs from `jobs` one at a time and "do" each one by
//!   sleeping `job.ms` milliseconds, then pushing `job.id` onto `done`.
//!   Stop when the channel closes (`stopped_by_cancel: false`) or when
//!   `token` is cancelled (`stopped_by_cancel: true`). Rules:
//!     * a job that has STARTED is always finished (don't race the sleep
//!       against the token);
//!     * between jobs, cancellation WINS over waiting jobs: if the token is
//!       already cancelled, start no new job, even if jobs are queued.
//!       (`tokio::select!` picks randomly among ready branches; `biased;`
//!       makes it check them in order.)
//! - `heartbeat`: count the ticks of `tokio::time::interval(period)` until
//!   the token is cancelled, and return the count. (The first tick is
//!   immediate, so cancelling after 350 ms with a 100 ms period gives 4.)
//!
//! The tests run on Tokio's paused clock, so they take no real time.
//!
//! Test: cargo test -p tokio_runtime --features solutions --bin sol06_cancellable_worker

use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{interval, sleep};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Copy)]
struct Job {
    id: u32,
    ms: u64,
}

#[derive(Debug, PartialEq)]
struct Report {
    done: Vec<u32>,
    stopped_by_cancel: bool,
}

async fn worker(mut jobs: mpsc::Receiver<Job>, token: CancellationToken) -> Report {
    let mut done = Vec::new();
    loop {
        // Only the WAIT for the next job races against the token. `biased;`
        // checks the token first, so cancellation wins over queued jobs.
        let job = tokio::select! {
            biased;
            _ = token.cancelled() => {
                return Report { done, stopped_by_cancel: true };
            }
            job = jobs.recv() => match job {
                Some(job) => job,
                None => return Report { done, stopped_by_cancel: false },
            },
        };
        // Outside the select!: a started job always runs to the end.
        sleep(Duration::from_millis(job.ms)).await;
        done.push(job.id);
    }
}

async fn heartbeat(period: Duration, token: CancellationToken) -> u32 {
    let mut ticks = 0;
    let mut every = interval(period);
    loop {
        tokio::select! {
            biased;
            _ = token.cancelled() => return ticks,
            _ = every.tick() => ticks += 1,
        }
    }
}

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel(8);
    for id in 1..=5 {
        tx.send(Job { id, ms: 100 }).await.unwrap();
    }
    let token = CancellationToken::new();
    let work = tokio::spawn(worker(rx, token.clone()));
    let beat = tokio::spawn(heartbeat(Duration::from_millis(100), token.clone()));
    sleep(Duration::from_millis(250)).await;
    token.cancel(); // "Ctrl+C"
    println!("{:?}", work.await.unwrap());
    println!("heartbeats: {}", beat.await.unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Instant;

    async fn queue(ids: std::ops::RangeInclusive<u32>) -> (mpsc::Sender<Job>, mpsc::Receiver<Job>) {
        let (tx, rx) = mpsc::channel(16);
        for id in ids {
            tx.send(Job { id, ms: 100 }).await.unwrap();
        }
        (tx, rx)
    }

    #[tokio::test(start_paused = true)]
    async fn stops_when_the_channel_closes() {
        let (tx, rx) = queue(1..=3).await;
        drop(tx);
        let start = Instant::now();
        let report = worker(rx, CancellationToken::new()).await;
        assert_eq!(
            report,
            Report {
                done: vec![1, 2, 3],
                stopped_by_cancel: false
            }
        );
        assert_eq!(start.elapsed(), Duration::from_millis(300));
    }

    #[tokio::test(start_paused = true)]
    async fn cancelling_finishes_the_current_job_only() {
        let (_tx, rx) = queue(1..=5).await;
        let token = CancellationToken::new();
        let canceller = token.clone();
        tokio::spawn(async move {
            sleep(Duration::from_millis(250)).await;
            canceller.cancel();
        });
        let start = Instant::now();
        let report = worker(rx, token).await;
        // job 3 started at 200 ms and is finished; jobs 4 and 5 never start
        assert_eq!(
            report,
            Report {
                done: vec![1, 2, 3],
                stopped_by_cancel: true
            }
        );
        assert_eq!(start.elapsed(), Duration::from_millis(300));
    }

    #[tokio::test(start_paused = true)]
    async fn cancellation_wins_over_waiting_jobs() {
        let (_tx, rx) = queue(1..=3).await;
        let token = CancellationToken::new();
        token.cancel();
        let report = worker(rx, token).await;
        assert_eq!(
            report,
            Report {
                done: vec![],
                stopped_by_cancel: true
            }
        );

        let token = CancellationToken::new();
        let canceller = token.clone();
        tokio::spawn(async move {
            sleep(Duration::from_millis(350)).await;
            canceller.cancel();
        });
        assert_eq!(heartbeat(Duration::from_millis(100), token).await, 4);
    }
}
