//! Streams with `tokio-stream`: adapters, turning channels and intervals
//! into streams, per-item timeouts, merging keyed streams with `StreamMap`,
//! and batching with `chunks_timeout`.
//! Run: cargo run -p tokio_runtime --example streams
//!
//! The clock is paused, so the times printed are exact. Each part prints
//! times counted from the start of that part.

use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{Instant, interval, interval_at, sleep};
use tokio_stream::wrappers::{IntervalStream, ReceiverStream};
use tokio_stream::{StreamExt, StreamMap};

/// A channel whose items arrive after the given delays (in ms).
fn arriving_after(delays: &'static [u64]) -> ReceiverStream<u64> {
    let (tx, rx) = mpsc::channel(8);
    tokio::spawn(async move {
        for (i, ms) in delays.iter().enumerate() {
            sleep(Duration::from_millis(*ms)).await;
            if tx.send(i as u64).await.is_err() {
                break;
            }
        }
    });
    ReceiverStream::new(rx)
}

fn ms(since: Instant) -> u128 {
    since.elapsed().as_millis()
}

#[tokio::main(flavor = "current_thread", start_paused = true)]
async fn main() {
    // 1. Iterator-style adapters, but every `next()` is awaited.
    let squares: Vec<u32> = tokio_stream::iter(1..=10)
        .filter(|n| n % 2 == 0)
        .map(|n| n * n)
        .take(3)
        .collect()
        .await;
    println!("1. even squares: {squares:?}");

    // 2. A channel receiver as a stream (ReceiverStream), read in a loop.
    let t0 = Instant::now();
    let mut items = arriving_after(&[10, 10, 10]);
    while let Some(n) = items.next().await {
        println!("2. [{:>3} ms] got item {n}", ms(t0));
    }

    // 3. An interval as a stream: a tick every 100 ms (the first at once).
    let t0 = Instant::now();
    let ticks: Vec<u128> = IntervalStream::new(interval(Duration::from_millis(100)))
        .take(3)
        .map(|_| ms(t0))
        .collect()
        .await;
    println!("3. ticks at {ticks:?} ms");

    // 4. A limit on the gap between items: a late item gives Err(Elapsed),
    //    and the stream carries on.
    //    `timeout` holds a timer, which must not move once polled, so pin
    //    the stream before calling `next()` on it.
    let t0 = Instant::now();
    let checked = arriving_after(&[10, 80, 10]).timeout(Duration::from_millis(50));
    tokio::pin!(checked);
    while let Some(item) = checked.next().await {
        println!("4. [{:>3} ms] {item:?}", ms(t0));
    }

    // 5. StreamMap: merge several streams, each item tagged with its key.
    let t0 = Instant::now();
    let mut map = StreamMap::new();
    let fast = interval_at(t0 + Duration::from_millis(10), Duration::from_millis(30));
    let slow = interval_at(t0 + Duration::from_millis(25), Duration::from_millis(50));
    map.insert("fast", IntervalStream::new(fast).take(3));
    map.insert("slow", IntervalStream::new(slow).take(2));
    while let Some((key, _tick)) = map.next().await {
        println!("5. [{:>3} ms] {key}", ms(t0));
    }

    // 6. chunks_timeout: batches of up to 3 items, or whatever arrived
    //    within 50 ms. Good for "save to the database in batches".
    let t0 = Instant::now();
    let batches =
        arriving_after(&[0, 10, 10, 10, 100, 10]).chunks_timeout(3, Duration::from_millis(50));
    tokio::pin!(batches);
    while let Some(batch) = batches.next().await {
        println!("6. [{:>3} ms] batch {batch:?}", ms(t0));
    }
}
