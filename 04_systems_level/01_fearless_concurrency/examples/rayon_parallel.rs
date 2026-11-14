//! Beyond the Book: data parallelism with the `rayon` crate. Change `iter()`
//! to `par_iter()` and rayon splits the work across a pool of threads, one
//! per CPU core, using work stealing.
//!
//! Run: cargo run -p fearless_concurrency --example rayon_parallel
//! (add `--release` to see the real speed-up)

use rayon::prelude::*;
use std::time::Instant;

fn is_prime(n: u64) -> bool {
    n >= 2
        && (2..)
            .take_while(|d| d * d <= n)
            .all(|d| !n.is_multiple_of(d))
}

fn main() {
    println!("rayon's pool has {} threads", rayon::current_num_threads());
    let limit = 200_000;

    // 1. The same iterator chain, sequential and parallel.
    let start = Instant::now();
    let sequential = (1..limit).filter(|&n| is_prime(n)).count();
    let seq_time = start.elapsed();

    let start = Instant::now();
    let parallel = (1..limit).into_par_iter().filter(|&n| is_prime(n)).count();
    let par_time = start.elapsed();
    assert_eq!(sequential, parallel);
    println!("1. {parallel} primes below {limit}: sequential {seq_time:?}, parallel {par_time:?}");

    // 2. map + sum / reduce over a Vec, in parallel.
    let words = vec!["fearless", "concurrency", "in", "rust", "is", "fun"];
    let total_len: usize = words.par_iter().map(|w| w.len()).sum();
    let longest = words
        .par_iter()
        .copied()
        .reduce(|| "", |a, b| if b.len() > a.len() { b } else { a });
    println!("2. total letters {total_len}, longest word {longest:?}");

    // 3. Parallel mutation in place: each element is visited by exactly one
    // thread, so no locks are needed.
    let mut squares: Vec<u64> = (0..10).collect();
    squares.par_iter_mut().for_each(|x| *x *= *x);
    println!("3. squares: {squares:?}");

    // 4. Parallel sort.
    let mut data: Vec<u64> = (0..100_000).map(|i| (i * 7_919) % 100_003).collect();
    data.par_sort_unstable();
    println!(
        "4. sorted {} numbers, first five {:?}",
        data.len(),
        &data[..5]
    );

    // 5. Fork-join: `rayon::join` runs two closures, possibly in parallel.
    fn sum(slice: &[u64]) -> u64 {
        if slice.len() <= 1_000 {
            return slice.iter().sum();
        }
        let (left, right) = slice.split_at(slice.len() / 2);
        let (a, b) = rayon::join(|| sum(left), || sum(right));
        a + b
    }
    println!("5. recursive parallel sum = {}", sum(&data));

    // 6. Your own pool, e.g. to limit rayon to 2 threads for one job.
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .expect("could not build the pool");
    let count = pool.install(|| {
        (1..50_000u64)
            .into_par_iter()
            .filter(|&n| is_prime(n))
            .count()
    });
    println!("6. a 2-thread pool counted {count} primes below 50000");
}
