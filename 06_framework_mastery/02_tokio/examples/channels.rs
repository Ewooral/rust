//! The four Tokio channels and when to use each: `mpsc` (a queue of work,
//! with back-pressure), `oneshot` (one reply), `broadcast` (everyone gets
//! every message) and `watch` (everyone sees the latest value).
//! Run: cargo run -p tokio_runtime --example channels
//!
//! The clock is paused, so the times printed are exact.

use std::time::Duration;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{broadcast, mpsc, oneshot, watch};
use tokio::time::{Instant, sleep};

#[tokio::main(flavor = "current_thread", start_paused = true)]
async fn main() {
    let start = Instant::now();
    let ms = move || start.elapsed().as_millis();

    // 1. mpsc: many producers, one consumer. BOUNDED: with room for 2,
    //    a fast producer must wait for the slow consumer (back-pressure).
    println!("1. mpsc::channel(2)");
    let (tx, mut rx) = mpsc::channel::<u32>(2);
    let producer = tokio::spawn(async move {
        for n in 1..=4 {
            tx.send(n).await.unwrap(); // waits while the channel is full
            println!("   [{:>3} ms] sent {n}", ms());
        }
        // tx is dropped here: the channel closes once it's empty
    });
    while let Some(n) = rx.recv().await {
        sleep(Duration::from_millis(100)).await; // a slow consumer
        println!("   [{:>3} ms] handled {n}", ms());
    }
    producer.await.unwrap();
    println!("   recv() returned None: every sender is gone");

    // 2. oneshot: exactly one value, typically a reply to a request.
    println!("2. oneshot");
    let (reply_tx, reply_rx) = oneshot::channel();
    tokio::spawn(async move {
        sleep(Duration::from_millis(10)).await;
        let _ = reply_tx.send("42 is the answer"); // Err if the receiver is gone
    });
    println!("   reply: {:?}", reply_rx.await);
    let (dropped_tx, dropped_rx) = oneshot::channel::<u32>();
    drop(dropped_tx);
    println!("   sender dropped: {:?}", dropped_rx.await);

    // 3. broadcast: every receiver gets every message. Each message is kept
    //    until all receivers have seen it, up to the capacity; a receiver
    //    that falls further behind gets Lagged(n) and skips ahead.
    println!("3. broadcast::channel(2)");
    let (tx, mut fast) = broadcast::channel::<u32>(2);
    let mut slow = tx.subscribe();
    for n in 1..=4 {
        tx.send(n).unwrap();
        println!("   fast got {:?}", fast.recv().await);
    }
    for _ in 0..3 {
        match slow.recv().await {
            Ok(n) => println!("   slow got {n}"),
            Err(RecvError::Lagged(missed)) => println!("   slow lagged: missed {missed}"),
            Err(RecvError::Closed) => break,
        }
    }

    // 4. watch: one value that changes; receivers see only the LATEST one.
    //    Perfect for configuration or "current status".
    println!("4. watch");
    let (tx, mut rx) = watch::channel("starting");
    let observer = tokio::spawn(async move {
        let mut seen = Vec::new();
        while rx.changed().await.is_ok() {
            seen.push(*rx.borrow_and_update());
        }
        seen // changed() errs once the sender is dropped
    });
    tx.send("loading").unwrap();
    tx.send("ready").unwrap(); // overwrites "loading" before the observer looks
    sleep(Duration::from_millis(10)).await;
    tx.send("stopping").unwrap();
    drop(tx);
    println!("   the observer saw {:?}", observer.await.unwrap());
}
