//! Exercise 5: build and consume streams with `futures::StreamExt`.
//!
//! Implement:
//!
//! - `even_squares`: keep only the even numbers, square them, collect.
//! - `total_until_error`: add up sensor readings, stopping at the first
//!   negative one (a sensor error); later readings are ignored.
//! - `countdown`: a stream that yields `from, from - 1, ..., 1`, pausing
//!   10 ms before each number, then ends.
//!
//! Hints: `filter` and `take_while` take closures that return a FUTURE, so
//! wrap a bool in `futures::future::ready(...)`; `map`, `collect().await`,
//! `fold`. For `countdown`, `stream::unfold(from, |n| async move { ... })`
//! returns `Some((item, next_state))` to continue and `None` to stop.
//!
//! Test: cargo test -p async_await --features exercises --bin ex05_stream_pipeline

#![allow(unused_variables, unused_imports)]

use futures::future::ready;
use futures::stream::{self, Stream, StreamExt};
use std::time::Duration;
use tokio::time::sleep;

async fn even_squares(numbers: impl Stream<Item = u64>) -> Vec<u64> {
    todo!()
}

async fn total_until_error(readings: impl Stream<Item = i32>) -> i32 {
    todo!()
}

fn countdown(from: u32) -> impl Stream<Item = u32> {
    // Replace this empty stream with your own.
    stream::empty()
}

#[tokio::main]
async fn main() {
    println!("{:?}", even_squares(stream::iter(1..=10)).await);
    println!("{}", total_until_error(stream::iter([5, 7, -1, 100])).await);
    let mut numbers = Box::pin(countdown(3));
    while let Some(n) = numbers.next().await {
        println!("{n}...");
    }
    println!("liftoff!");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Instant;

    #[tokio::test]
    async fn squares_of_the_even_numbers() {
        assert_eq!(even_squares(stream::iter(1..=6)).await, [4, 16, 36]);
        assert!(even_squares(stream::iter([1, 3, 5])).await.is_empty());
    }

    #[tokio::test]
    async fn stops_at_the_first_error() {
        assert_eq!(total_until_error(stream::iter([5, 7, -1, 100])).await, 12);
        assert_eq!(total_until_error(stream::iter([1, 2, 3])).await, 6);
        assert_eq!(total_until_error(stream::iter([-4, 10])).await, 0);
    }

    #[tokio::test(start_paused = true)]
    async fn countdown_counts_down_with_pauses() {
        let start = Instant::now();
        let numbers: Vec<u32> = countdown(3).collect().await;
        assert_eq!(numbers, [3, 2, 1]);
        assert!(
            start.elapsed() >= Duration::from_millis(30),
            "10 ms before each number"
        );
        assert!(countdown(0).collect::<Vec<_>>().await.is_empty());
    }
}
