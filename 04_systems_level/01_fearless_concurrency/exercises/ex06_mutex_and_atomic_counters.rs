//! Exercise 6: the same shared counter, with a lock and without one.
//!
//! Start `threads` threads; each adds 1 to a shared counter `increments`
//! times. Return the final count, which must be exactly
//! `threads * increments` every time.
//!
//! - `count_with_mutex`: `Arc<Mutex<usize>>` and `thread::spawn` (Book 16.3).
//! - `count_with_atomic`: `Arc<AtomicUsize>` and `fetch_add(1, Ordering::Relaxed)`,
//!   no lock at all (Beyond the Book).
//!
//! The tests are written; make them pass. Then run it with `--release` and
//! compare the timings `main` prints.
//!
//! Test: cargo test -p fearless_concurrency --features exercises --bin ex06_mutex_and_atomic_counters

#![allow(unused_variables, unused_imports)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

fn count_with_mutex(threads: usize, increments: usize) -> usize {
    todo!()
}

fn count_with_atomic(threads: usize, increments: usize) -> usize {
    todo!()
}

fn main() {
    let start = Instant::now();
    let a = count_with_mutex(8, 100_000);
    println!("mutex:  {a} in {:?}", start.elapsed());
    let start = Instant::now();
    let b = count_with_atomic(8, 100_000);
    println!("atomic: {b} in {:?}", start.elapsed());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutex_counter_is_exact() {
        assert_eq!(count_with_mutex(8, 1_000), 8_000);
        assert_eq!(count_with_mutex(1, 5), 5);
    }

    #[test]
    fn atomic_counter_is_exact() {
        assert_eq!(count_with_atomic(8, 1_000), 8_000);
        assert_eq!(count_with_atomic(3, 7), 21);
    }

    #[test]
    fn no_threads_means_zero() {
        assert_eq!(count_with_mutex(0, 1_000), 0);
        assert_eq!(count_with_atomic(0, 1_000), 0);
    }
}
