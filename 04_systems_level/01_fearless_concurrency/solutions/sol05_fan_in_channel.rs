//! Solution to exercise 5: many producers, one consumer ("fan-in").
//!
//! Implement `squares_from_workers`: start ONE thread per batch. Each thread
//! sends the square of every number in its batch down a shared channel
//! (each thread gets its own clone of the `Sender`). The calling thread
//! collects everything that arrives and returns it SORTED (threads finish
//! in any order). The tests are written; make them pass.
//!
//! Hints: `mpsc::channel()`, `tx.clone()`, `thread::spawn(move || ...)`,
//! `rx.iter().collect()` or `for n in rx`. If a test HANGS, the receiver is
//! still waiting for a sender that never goes away: did you `drop(tx)`?
//!
//! Test: cargo test -p fearless_concurrency --features solutions --bin sol05_fan_in_channel

use std::sync::mpsc;
use std::thread;

fn squares_from_workers(batches: Vec<Vec<u64>>) -> Vec<u64> {
    let (tx, rx) = mpsc::channel();
    for batch in batches {
        let tx = tx.clone(); // one Sender per producer thread
        thread::spawn(move || {
            for n in batch {
                tx.send(n * n).unwrap();
            }
        }); // the thread ends and drops its Sender
    }
    // Drop the original Sender too, or `rx` would wait for it forever.
    drop(tx);
    let mut squares: Vec<u64> = rx.iter().collect();
    squares.sort_unstable();
    squares
}

fn main() {
    let batches = vec![vec![1, 2, 3], vec![4, 5], vec![6]];
    println!("{:?}", squares_from_workers(batches));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn squares_every_number_from_every_batch() {
        let batches = vec![vec![3, 1], vec![2], vec![5, 4]];
        assert_eq!(squares_from_workers(batches), [1, 4, 9, 16, 25]);
    }

    #[test]
    fn no_batches_or_empty_batches() {
        assert!(squares_from_workers(vec![]).is_empty());
        assert!(squares_from_workers(vec![vec![], vec![]]).is_empty());
    }

    #[test]
    fn a_hundred_producers() {
        let batches: Vec<Vec<u64>> = (0..100).map(|_| (0..10).collect()).collect();
        let squares = squares_from_workers(batches);
        assert_eq!(squares.len(), 1_000);
        assert_eq!(squares.iter().sum::<u64>(), 100 * 285);
    }
}
