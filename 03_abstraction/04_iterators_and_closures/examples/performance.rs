//! Book 13.4: loops vs iterators. Iterators are a "zero-cost abstraction":
//! the compiler turns the high-level chain into the same (or better)
//! machine code as the hand-written loop.
//!
//! Measure in RELEASE mode; debug builds skip the optimisations entirely:
//!   cargo run -p iterators_and_closures --example performance --release

use std::hint::black_box;
use std::time::Instant;

/// Search with a for loop.
fn search_loop<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let mut results = Vec::new();
    for line in contents.lines() {
        if line.contains(query) {
            results.push(line);
        }
    }
    results
}

/// The same search with an iterator chain.
fn search_iter<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    contents
        .lines()
        .filter(|line| line.contains(query))
        .collect()
}

/// The Book's example from an audio decoder: a "linear prediction" sum,
/// written with zip, map and sum. In release mode the compiler unrolls the
/// 12-step loop and keeps the coefficients in registers.
fn predict(buffer: &[i32], coefficients: &[i64; 12], qlp_shift: i16) -> Vec<i32> {
    let mut out = buffer.to_vec();
    for i in 12..buffer.len() {
        let prediction = coefficients
            .iter()
            .zip(&buffer[i - 12..i])
            .map(|(&c, &s)| c * s as i64)
            .sum::<i64>()
            >> qlp_shift;
        out[i] = buffer[i] - prediction as i32;
    }
    out
}

fn time<F: FnMut() -> usize>(label: &str, mut f: F) {
    let start = Instant::now();
    let mut total = 0;
    for _ in 0..20 {
        total += f();
    }
    println!("{label:>10}: {:>8.2?} (result {total})", start.elapsed());
}

fn main() {
    let text = "the quick brown fox jumps over the lazy dog\n".repeat(200_000);
    time("for loop", || {
        search_loop(black_box("lazy"), black_box(&text)).len()
    });
    time("iterator", || {
        search_iter(black_box("lazy"), black_box(&text)).len()
    });

    let buffer: Vec<i32> = (0..1000).map(|x| (x * 37) % 101).collect();
    let coefficients = [1, -2, 3, -4, 5, -6, 7, -8, 9, -10, 11, -12];
    let decoded = predict(&buffer, &coefficients, 2);
    println!(
        "decoded {} samples, first predicted: {}",
        decoded.len(),
        decoded[12]
    );
    println!("(timings vary with machine load: run it a few times, always with --release)");
}
