//! Measure before you optimise: three traps when timing code by hand.
//!
//! Run: cargo run -p performance --example measure_first --release
//! Then without --release, and compare: the same code, very different numbers.

use std::fmt::Write as _;
use std::hint::black_box;
use std::time::Instant;

use performance::timing::{build_kind, median_time};

const N: u64 = 2_000_000;

/// A hash of the numbers 0..n. Each step depends on the one before, so the
/// optimiser can't skip ahead or replace the loop with a formula (which it
/// happily does for simpler loops, such as a sum of squares).
fn checksum(n: u64) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for i in 0..n {
        hash = (hash ^ i).wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

fn main() {
    println!("{}\n", build_kind());

    // Trap 1: the optimiser deletes work whose result nobody uses. In a
    // release build the first measurement can be ~0 ns: it never ran.
    let start = Instant::now();
    let _unused = checksum(N);
    let deleted = start.elapsed();
    let start = Instant::now();
    black_box(checksum(black_box(N))); // "you can't know what this is"
    let kept = start.elapsed();
    println!("1. result unused:             {deleted:>10.2?}  (maybe never ran!)");
    println!("   with std::hint::black_box: {kept:>10.2?}\n");

    // Trap 2: one run is one sample. The first run is often the slowest
    // (cold caches, memory being mapped in); others are disturbed by the OS.
    let mut runs = Vec::new();
    for _ in 0..5 {
        let start = Instant::now();
        black_box(checksum(black_box(N)));
        runs.push(start.elapsed());
    }
    println!("2. five runs of the same thing: {runs:.2?}");
    println!(
        "   median: {:.2?}  <- report this (or let criterion do it)\n",
        median_time(5, || checksum(black_box(N)))
    );

    // Trap 3: guessing where the time goes. Which step of this job is slow?
    let numbers: Vec<u64> = (0..200_000).map(|i| (i * 7919) % 100_003).collect();
    let sort = median_time(5, || {
        let mut v = numbers.clone();
        v.sort_unstable();
        v
    });
    let format = median_time(5, || {
        let mut out = String::with_capacity(numbers.len() * 7);
        for n in &numbers {
            let _ = write!(out, "{n},");
        }
        out
    });
    let sum = median_time(5, || numbers.iter().sum::<u64>());
    println!("3. one job, three steps (200 000 numbers):");
    println!("   sort   {sort:>10.2?}");
    println!("   format {format:>10.2?}");
    println!("   sum    {sum:>10.2?}");
    println!("   Optimise the biggest number, not the step you suspected.");
}
