//! `select!`, timeouts, deadlines and intervals.
//! Run: cargo run -p tokio_runtime --example select_and_time
//!
//! The clock is paused, so the times printed are exact.

use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{Instant, MissedTickBehavior, interval, sleep, sleep_until, timeout};

#[tokio::main(flavor = "current_thread", start_paused = true)]
async fn main() {
    let start = Instant::now();
    let ms = move || start.elapsed().as_millis();

    // 1. select!: wait on several futures, run the branch of the FIRST one
    //    to finish, and DROP (cancel) the others.
    let winner = tokio::select! {
        _ = sleep(Duration::from_millis(50)) => "the 50 ms sleep",
        _ = sleep(Duration::from_millis(20)) => "the 20 ms sleep",
    };
    println!("1. [{:>3} ms] select! picked {winner}", ms());

    // 2. Branches are checked in RANDOM order when several are ready at
    //    once (fairness). `biased;` checks them top to bottom instead.
    let (tx, mut rx) = mpsc::channel(4);
    tx.send("message").await.unwrap();
    let picked = tokio::select! {
        biased;
        _ = std::future::ready(()) => "the always-ready branch",
        Some(m) = rx.recv() => m,
    };
    println!("2. biased; with two ready branches picked {picked}");

    // 3. The losing `rx.recv()` was dropped, but the message wasn't lost:
    //    `recv` is CANCEL SAFE (the message stays in the channel until a
    //    recv() completes). Patterns and `else`: a branch whose pattern
    //    doesn't match is disabled; `else` runs when all are disabled.
    drop(tx); // once empty, rx.recv() returns None
    for _ in 0..2 {
        let outcome = tokio::select! {
            Some(m) = rx.recv() => format!("still in the channel: {m:?}"),
            else => "else: the channel is closed".to_string(),
        };
        println!("3. {outcome}");
    }

    // 4. timeout(): give a future a time limit.
    let slow = timeout(Duration::from_millis(30), sleep(Duration::from_secs(5))).await;
    let quick = timeout(Duration::from_millis(30), async { 7 }).await;
    println!(
        "4. [{:>3} ms] timeout: slow -> {slow:?}, quick -> {quick:?}",
        ms()
    );

    // 5. A deadline for a whole loop of work: create ONE sleep, pin it, and
    //    poll it by `&mut` in every round. (Writing `sleep(..)` inside the
    //    loop would restart the clock every time round.)
    let (tx, mut rx) = mpsc::channel(16);
    tokio::spawn(async move {
        for n in 0.. {
            sleep(Duration::from_millis(40)).await;
            if tx.send(n).await.is_err() {
                break;
            }
        }
    });
    let deadline = sleep_until(Instant::now() + Duration::from_millis(150));
    tokio::pin!(deadline); // `&mut deadline` needs it pinned
    let mut got = Vec::new();
    loop {
        tokio::select! {
            _ = &mut deadline => break,
            Some(n) = rx.recv() => got.push(n),
        }
    }
    println!("5. [{:>3} ms] collected {got:?} before the deadline", ms());

    // 6. interval: a tick every period. The FIRST tick is immediate.
    //    If a tick is missed (the loop was busy), MissedTickBehavior decides:
    //    Burst (the default) catches up at once, Delay shifts the schedule,
    //    Skip jumps to the next multiple of the period.
    for behavior in [
        MissedTickBehavior::Burst,
        MissedTickBehavior::Delay,
        MissedTickBehavior::Skip,
    ] {
        let t0 = Instant::now();
        let mut ticker = interval(Duration::from_millis(100));
        ticker.set_missed_tick_behavior(behavior);
        let mut times = Vec::new();
        for i in 0..5 {
            ticker.tick().await;
            times.push(t0.elapsed().as_millis());
            if i == 1 {
                sleep(Duration::from_millis(250)).await; // busy: misses ticks
            }
        }
        println!("6. {behavior:?}: ticks at {times:?} ms");
    }
}
