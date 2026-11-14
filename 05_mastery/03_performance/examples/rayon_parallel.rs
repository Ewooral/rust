//! Parallelism with rayon: when it pays, and when it doesn't.
//!
//! Run: cargo run -p performance --example rayon_parallel --release
//!
//! rayon was introduced in 04_systems_level/01_fearless_concurrency. Here we
//! only ask the performance question: is it actually faster?

use rayon::prelude::*;

use performance::timing::{build_kind, median_time};

/// Deliberately expensive per item: up to a few hundred steps of arithmetic.
fn collatz_steps(mut n: u64) -> u32 {
    let mut steps = 0;
    while n != 1 {
        n = if n.is_multiple_of(2) {
            n / 2
        } else {
            3 * n + 1
        };
        steps += 1;
    }
    steps
}

fn main() {
    println!("{}", build_kind());
    println!("rayon will use {} threads\n", rayon::current_num_threads());

    // 1. Plenty of work per item: parallel wins.
    let numbers: Vec<u64> = (1..200_000).collect();
    let seq = median_time(5, || numbers.iter().map(|&n| collatz_steps(n)).max());
    let par = median_time(5, || numbers.par_iter().map(|&n| collatz_steps(n)).max());
    println!("heavy work: Collatz steps for 200 000 numbers");
    println!("  iter()     {seq:>10.2?}");
    println!(
        "  par_iter() {par:>10.2?}  ({:.1}x faster)\n",
        seq.as_secs_f64() / par.as_secs_f64()
    );

    // 2. Almost no work per item: sharing the job out costs more than it saves.
    let small: Vec<u64> = (0..1_000).collect();
    let seq = median_time(9, || small.iter().sum::<u64>());
    let par = median_time(9, || small.par_iter().sum::<u64>());
    println!("tiny work: the sum of 1 000 numbers");
    println!("  iter()     {seq:>10.2?}");
    println!("  par_iter() {par:>10.2?}  <- usually SLOWER: the overhead wins");
}
