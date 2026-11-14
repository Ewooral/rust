//! Solution to exercise 4: split a sum across threads with `thread::scope`.
//!
//! Implement `parallel_sum`: cut `numbers` into `threads` chunks, sum each
//! chunk on its own scoped thread, then add up the partial sums. Treat
//! `threads == 0` as 1. The tests are written; make them pass.
//!
//! Hints: `numbers.len().div_ceil(threads)` is the chunk size (but
//! `chunks(0)` panics, so keep it at least 1); `numbers.chunks(size)`;
//! `thread::scope(|s| ...)`, `s.spawn(move || ...)` and `handle.join()`.
//! Scoped threads may borrow `numbers`, so no `Arc` or cloning is needed.
//!
//! Test: cargo test -p fearless_concurrency --features solutions --bin sol04_parallel_sum

use std::thread;

fn parallel_sum(numbers: &[u64], threads: usize) -> u64 {
    let threads = threads.max(1);
    // At least 1, because `chunks(0)` panics (an empty slice gives size 0).
    let chunk_size = numbers.len().div_ceil(threads).max(1);
    thread::scope(|s| {
        let handles: Vec<_> = numbers
            .chunks(chunk_size)
            .map(|chunk| s.spawn(move || chunk.iter().sum::<u64>()))
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).sum()
    })
}

fn main() {
    let numbers: Vec<u64> = (1..=1_000_000).collect();
    for threads in [1, 2, 4, 8] {
        println!("{threads} threads: {}", parallel_sum(&numbers, threads));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_sequential_sum() {
        let numbers: Vec<u64> = (1..=10_000).collect();
        let expected: u64 = numbers.iter().sum();
        for threads in [1, 2, 3, 7, 16] {
            assert_eq!(
                parallel_sum(&numbers, threads),
                expected,
                "{threads} threads"
            );
        }
    }

    #[test]
    fn more_threads_than_numbers() {
        assert_eq!(parallel_sum(&[5, 6, 7], 10), 18);
    }

    #[test]
    fn empty_slice_and_zero_threads() {
        assert_eq!(parallel_sum(&[], 4), 0);
        assert_eq!(parallel_sum(&[1, 2, 3], 0), 6);
    }
}
