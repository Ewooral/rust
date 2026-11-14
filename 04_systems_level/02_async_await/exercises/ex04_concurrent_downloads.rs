//! Exercise 4: run futures concurrently, with timeouts, and race them.
//!
//! `download` pretends to fetch a file. Implement:
//!
//! - `download_all`: download every job CONCURRENTLY (not one after
//!   another). Each job may take at most `limit`: a job that finishes in
//!   time gives `Ok(text)`, a slower one gives `Err(name)`. Return the
//!   results in the same order as `jobs`.
//! - `fastest`: race every job and return the name of the one that finishes
//!   first (`None` if there are no jobs).
//!
//! Hints: `futures::future::join_all`, `tokio::time::timeout`, and for the
//! race `futures::future::select_all` (box the futures with `Box::pin`) or
//! `FuturesUnordered`. The tests use Tokio's paused clock, so they run
//! instantly, yet `elapsed()` still shows whether you ran things
//! concurrently.
//!
//! Test: cargo test -p async_await --features exercises --bin ex04_concurrent_downloads

#![allow(unused_variables, unused_imports, dead_code)]

use futures::future::{join_all, select_all};
use std::time::Duration;
use tokio::time::{sleep, timeout};

#[derive(Debug, Clone, Copy)]
struct Job {
    name: &'static str,
    ms: u64,
}

async fn download(job: Job) -> String {
    sleep(Duration::from_millis(job.ms)).await;
    format!("{} ({} ms)", job.name, job.ms)
}

async fn download_all(jobs: &[Job], limit: Duration) -> Vec<Result<String, &'static str>> {
    todo!()
}

async fn fastest(jobs: &[Job]) -> Option<&'static str> {
    todo!()
}

#[tokio::main]
async fn main() {
    let jobs = [
        Job {
            name: "cat.png",
            ms: 30,
        },
        Job {
            name: "film.mp4",
            ms: 500,
        },
        Job {
            name: "notes.txt",
            ms: 10,
        },
    ];
    for result in download_all(&jobs, Duration::from_millis(100)).await {
        println!("{result:?}");
    }
    println!("fastest: {:?}", fastest(&jobs).await);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Instant;

    fn jobs() -> Vec<Job> {
        vec![
            Job { name: "a", ms: 30 },
            Job { name: "b", ms: 10 },
            Job { name: "c", ms: 20 },
        ]
    }

    #[tokio::test(start_paused = true)]
    async fn downloads_run_at_the_same_time_and_keep_their_order() {
        let start = Instant::now();
        let results = download_all(&jobs(), Duration::from_millis(100)).await;
        assert_eq!(
            results,
            [
                Ok("a (30 ms)".to_string()),
                Ok("b (10 ms)".to_string()),
                Ok("c (20 ms)".to_string())
            ]
        );
        // One after another would take 60 ms; together, as long as the slowest.
        assert!(
            start.elapsed() < Duration::from_millis(50),
            "took {:?}",
            start.elapsed()
        );
    }

    #[tokio::test(start_paused = true)]
    async fn slow_downloads_time_out() {
        let jobs = [
            Job {
                name: "huge",
                ms: 5_000,
            },
            Job {
                name: "tiny",
                ms: 5,
            },
        ];
        let start = Instant::now();
        let results = download_all(&jobs, Duration::from_millis(100)).await;
        assert_eq!(results, [Err("huge"), Ok("tiny (5 ms)".to_string())]);
        assert!(
            start.elapsed() < Duration::from_millis(200),
            "the timeout must stop the wait"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn the_fastest_job_wins_the_race() {
        assert_eq!(fastest(&jobs()).await, Some("b"));
        assert_eq!(fastest(&[]).await, None);
    }
}
