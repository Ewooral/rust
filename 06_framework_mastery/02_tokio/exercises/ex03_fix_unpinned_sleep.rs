//! Exercise 3: make it compile (E0277: `PhantomPinned` cannot be unpinned).
//!
//! The loop should collect messages for 100 ms in total, then stop. The
//! deadline is ONE `Sleep`, created before the loop and polled by `&mut`
//! in every round, so its clock doesn't restart each time round.
//!
//! But polling a future through `&mut` needs it to be `Unpin`, and Tokio's
//! `Sleep` isn't: once it has been polled, it must not move in memory. The
//! fix is to PIN it in place first: `tokio::pin!(deadline);` on the line
//! after creating it (or create it with `std::pin::pin!(sleep(..))`).
//!
//! Run: cargo run -p tokio_runtime --features exercises --bin ex03_fix_unpinned_sleep

use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{Instant, sleep};

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(16);
    tokio::spawn(async move {
        for n in 1.. {
            sleep(Duration::from_millis(30)).await;
            if tx.send(n).await.is_err() {
                break; // the receiver has gone
            }
        }
    });

    let start = Instant::now();
    let mut deadline = sleep(Duration::from_millis(100));
    let mut got = Vec::new();
    loop {
        tokio::select! {
            _ = &mut deadline => break,
            Some(n) = rx.recv() => got.push(n),
        }
    }
    println!(
        "collected {got:?} in about {} ms",
        start.elapsed().as_millis() / 50 * 50
    );
}
