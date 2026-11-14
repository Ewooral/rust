//! Book 17.4 (streams: futures in sequence), then Beyond the Book: async
//! streams with the `futures` crate.
//!
//! A stream is the async version of an iterator: `next().await` gives
//! `Some(item)` when the next item is READY, and `None` at the end.
//!
//! Two crates provide a `StreamExt` trait full of adapters. Their method
//! names overlap (`map`, `filter`, `next`, ...), so import ONE per module:
//!
//! - `book` uses `tokio_stream::StreamExt`, which is what the Book's `trpl`
//!   re-exports (it has `timeout`, `throttle` and `merge`).
//! - `futures_crate` uses `futures::StreamExt` (it has `then`,
//!   `buffer_unordered`, `fold` and friends).
//!
//! Run: cargo run -p async_await --example streams

#[tokio::main]
async fn main() {
    book::run().await;
    futures_crate::run().await;
}

mod book {
    use std::pin::pin;
    use std::time::Duration;
    use tokio::sync::mpsc;
    use tokio::time::{interval, sleep};
    use tokio_stream::wrappers::{IntervalStream, UnboundedReceiverStream};
    use tokio_stream::{Stream, StreamExt};

    pub async fn run() {
        println!("=== Book 17.4: streams ===");

        // 1. A stream from an iterator (the Book's `trpl::stream_from_iter`),
        // with iterator-style adapters. `next()` needs `.await`.
        let values = 1..=10;
        let mut stream = tokio_stream::iter(values)
            .map(|n| n * 2)
            .filter(|n| n % 3 == 0);
        while let Some(value) = stream.next().await {
            println!("  1. a multiple of 3, doubled: {value}");
        }

        // 2. Composing streams: messages arrive from a channel, and each
        // gets a deadline. A message that's late shows up as an error, but
        // the stream carries on.
        let mut messages = pin!(get_messages().timeout(Duration::from_millis(30)));
        while let Some(result) = messages.next().await {
            match result {
                Ok(message) => println!("  2. {message}"),
                Err(reason) => println!("  2. problem: {reason}"),
            }
        }

        // 3. Merging two streams, and throttling: a fast interval stream
        // limited to one item per 20 ms, merged with the messages, stopped
        // after 10 items. `take` makes an endless stream finish.
        let messages = get_messages().timeout(Duration::from_millis(30));
        let intervals = get_intervals()
            .map(|count| format!("interval #{count}"))
            .throttle(Duration::from_millis(20))
            .timeout(Duration::from_millis(30));
        let mut merged = pin!(messages.merge(intervals).take(10));
        while let Some(result) = merged.next().await {
            match result {
                Ok(item) => println!("  3. {item}"),
                Err(reason) => println!("  3. problem: {reason}"),
            }
        }
    }

    /// The Book's `get_messages`: a task sends letters with growing pauses
    /// (every other one is too slow for the 30 ms timeout); the receiver
    /// becomes a stream with `UnboundedReceiverStream` (`trpl::ReceiverStream`).
    fn get_messages() -> impl Stream<Item = String> {
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            for (index, letter) in ["a", "b", "c", "d"].into_iter().enumerate() {
                let ms = if index % 2 == 0 { 5 } else { 80 };
                sleep(Duration::from_millis(ms)).await;
                if tx.send(format!("message: '{letter}'")).is_err() {
                    break; // the stream was dropped: nobody is listening
                }
            }
        });
        UnboundedReceiverStream::new(rx)
    }

    /// An endless stream of counts, one every 2 ms (`trpl::IntervalStream`).
    fn get_intervals() -> impl Stream<Item = u32> {
        let mut count = 0;
        IntervalStream::new(interval(Duration::from_millis(2))).map(move |_| {
            count += 1;
            count
        })
    }
}

mod futures_crate {
    use futures::stream::{self, FuturesUnordered, StreamExt};
    use std::time::{Duration, Instant};
    use tokio::time::sleep;

    async fn download(page: u32) -> String {
        sleep(Duration::from_millis(10 * u64::from(5 - page))).await;
        format!("page {page}")
    }

    pub async fn run() {
        println!("=== Beyond the Book: the futures crate's streams ===");

        // 1. `then` is an ASYNC map: one future per item, one at a time.
        let start = Instant::now();
        let pages: Vec<String> = stream::iter(1..=4).then(download).collect().await;
        println!(
            "  1. then: {pages:?} in {:?} (one after another)",
            start.elapsed()
        );

        // 2. `buffer_unordered(n)`: run up to n futures AT ONCE and yield
        // results as they finish. (`buffered(n)` keeps the input order.)
        let start = Instant::now();
        let pages: Vec<String> = stream::iter(1..=4)
            .map(download)
            .buffer_unordered(3)
            .collect()
            .await;
        println!(
            "  2. buffer_unordered(3): {pages:?} in {:?}",
            start.elapsed()
        );

        // 3. `unfold`: build a stream from a state, like a paginated API
        // that hands you a "next page" cursor until there are no more.
        let cursors = stream::unfold(Some(1u32), |cursor| async move {
            let page = cursor?; // None: the stream ends
            sleep(Duration::from_millis(2)).await;
            let next = if page < 3 { Some(page + 1) } else { None };
            Some((format!("results page {page}"), next))
        });
        let all: Vec<String> = cursors.collect().await;
        println!("  3. unfold: {all:?}");

        // 4. `FuturesUnordered`: a set of futures that is itself a stream of
        // their results, in the order they FINISH.
        let mut in_flight: FuturesUnordered<_> = (1..=4).map(download).collect();
        let mut order = Vec::new();
        while let Some(page) = in_flight.next().await {
            order.push(page);
        }
        println!("  4. FuturesUnordered finish order: {order:?}");

        // 5. `fold`, `take_while`, `chunks` and `select` (merge two streams).
        let total = stream::iter(1..=10)
            .fold(0, |sum, n| async move { sum + n })
            .await;
        let small: Vec<u32> = stream::iter(1..=10)
            .take_while(|n| futures::future::ready(*n < 4))
            .collect()
            .await;
        let chunks: Vec<Vec<u32>> = stream::iter(1..=7).chunks(3).collect().await;
        let merged: Vec<&str> =
            stream::select(stream::iter(["L1", "L2"]), stream::iter(["R1", "R2"]))
                .collect()
                .await;
        println!(
            "  5. fold = {total}, take_while = {small:?}, chunks = {chunks:?}, select = {merged:?}"
        );
    }
}
