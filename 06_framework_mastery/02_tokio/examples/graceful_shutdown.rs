//! Graceful shutdown with `tokio-util`: `CancellationToken` (tell every task
//! to stop), child tokens, `run_until_cancelled`, `DropGuard`, and
//! `TaskTracker` (wait until every task has finished).
//! Run: cargo run -p tokio_runtime --example graceful_shutdown
//!
//! A real program cancels the token when Ctrl+C arrives:
//!   tokio::signal::ctrl_c().await?; token.cancel();
//! Here a timer plays Ctrl+C, so the example ends by itself. The clock is
//! paused, so the times printed are exact.

use std::time::Duration;
use tokio::time::{Instant, interval, sleep};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

#[tokio::main(flavor = "current_thread", start_paused = true)]
async fn main() {
    let start = Instant::now();
    let ms = move || start.elapsed().as_millis();

    // 1. Workers that check the token in a select! loop, and a tracker
    //    that knows when they've all finished.
    let token = CancellationToken::new();
    let tracker = TaskTracker::new();
    for id in 1..=3 {
        let token = token.clone();
        tracker.spawn(async move {
            let mut jobs = 0;
            let mut every = interval(Duration::from_millis(45 * id));
            loop {
                tokio::select! {
                    _ = token.cancelled() => break,
                    _ = every.tick() => jobs += 1, // "one job done"
                }
            }
            sleep(Duration::from_millis(10 * id)).await; // tidy up: flush, save...
            println!("   [{:>3} ms] worker {id} stopped after {jobs} jobs", ms());
        });
    }
    tracker.close(); // no more tasks will be added

    sleep(Duration::from_millis(200)).await;
    println!("1. [{:>3} ms] \"Ctrl+C\": cancel the token", ms());
    token.cancel();
    tracker.wait().await; // every worker has tidied up
    println!(
        "   [{:>3} ms] tracker.wait() returned: no tasks left: {}",
        ms(),
        tracker.is_empty()
    );

    // 2. Child tokens: cancelling a parent cancels its children, but not
    //    the other way round. Give each part of a program its own child.
    let parent = CancellationToken::new();
    let child_a = parent.child_token();
    let child_b = parent.child_token();
    child_a.cancel();
    println!(
        "2. after child_a.cancel(): parent {}, child_b {}",
        parent.is_cancelled(),
        child_b.is_cancelled()
    );
    parent.cancel();
    println!(
        "   after parent.cancel(): child_b {}",
        child_b.is_cancelled()
    );

    // 3. run_until_cancelled: Some(output) if the future finished first,
    //    None if the token was cancelled first (the future is dropped).
    let token = CancellationToken::new();
    let quick = token.run_until_cancelled(async { "done" }).await;
    let trigger = token.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(50)).await;
        trigger.cancel();
    });
    let slow = token
        .run_until_cancelled(sleep(Duration::from_secs(10)))
        .await;
    println!("3. run_until_cancelled: quick {quick:?}, slow {slow:?}");

    // 4. DropGuard: cancel automatically when the guard is dropped, for
    //    example when a function returns early or panics.
    let token = CancellationToken::new();
    {
        let _guard = token.clone().drop_guard();
        // ... work that might return early ...
    } // guard dropped here
    println!(
        "4. after the DropGuard was dropped: cancelled {}",
        token.is_cancelled()
    );
}
