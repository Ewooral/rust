//! Why `--release` exists: the same code, timed in both profiles.
//! Run: cargo run -p hello_world --example debug_vs_release
//! Then: cargo run -p hello_world --example debug_vs_release --release
//!
//! Expect the release run to be several times faster. Exact numbers depend
//! on your machine.

use std::hint::black_box;
use std::time::Instant;

fn main() {
    let start = Instant::now();

    // Add up the numbers 0 to 99,999,999. `black_box` stops the optimiser from
    // working out the answer at compile time, so the loop really runs.
    let mut total: u64 = 0;
    for n in 0..black_box(100_000_000u64) {
        total = total.wrapping_add(black_box(n));
    }

    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    println!("total = {total}");
    println!("{profile} build took {:?}", start.elapsed());
}
