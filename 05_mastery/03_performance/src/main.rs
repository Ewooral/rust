//! Mini project: a log-analysis pipeline, optimised one measured step at a time.
//!
//! The same report is computed by four versions (see `src/pipeline.rs`).
//! This program checks that they agree, then times each one and counts its
//! heap allocations, so you can see what every change bought.
//!
//! Run:   cargo run -p performance --release              (200 000 lines)
//!        cargo run -p performance --release -- 1000000   (a million)
//! Bench: cargo bench -p performance --bench pipeline     (the careful measurement)
//! Test:  cargo test -p performance

use std::process;
use std::time::Duration;

use performance::alloc_counter::{CountingAlloc, count_allocations_all_threads};
use performance::timing::median_time;
use performance::{logs, pipeline};

// Count every allocation this program makes (see src/alloc_counter.rs).
#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

const RUNS: usize = 5;

fn main() {
    let lines = match std::env::args().nth(1).map(|a| a.parse::<usize>()) {
        // A debug build is 10-50x slower, so it gets a smaller default.
        None if cfg!(debug_assertions) => 10_000,
        None => 200_000,
        Some(Ok(n)) if n > 0 => n,
        _ => {
            eprintln!("usage: performance [number_of_lines]");
            process::exit(2);
        }
    };

    if cfg!(debug_assertions) {
        println!("WARNING: debug build. These timings mean nothing: run with --release.\n");
    }

    let log = logs::generate(lines, 42);
    let megabytes = log.len() as f64 / 1_000_000.0;
    println!("{lines} log lines, {megabytes:.1} MB, {RUNS} runs each (median)\n");
    println!(
        "{:<12} {:>10} {:>9} {:>12} {:>9}",
        "version", "time", "speed-up", "allocations", "MB/s"
    );

    let mut first: Option<(Duration, pipeline::Report)> = None;
    for (name, analyse) in pipeline::VERSIONS {
        // 1. Correctness first: every version must give the same report.
        let (report, allocs) = count_allocations_all_threads(|| analyse(&log));
        if let Some((_, expected)) = &first
            && report != *expected
        {
            eprintln!("{name} gives a DIFFERENT report: an optimisation broke something");
            process::exit(1);
        }

        // 2. Then speed.
        // (median_time also black_boxes the result, so no work is skipped.)
        let time = median_time(RUNS, || analyse(std::hint::black_box(&log)));
        let baseline = first.get_or_insert((time, report)).0;
        println!(
            "{:<12} {:>10.2?} {:>8.1}x {:>12} {:>9.0}",
            name,
            time,
            baseline.as_secs_f64() / time.as_secs_f64(),
            allocs.allocations,
            megabytes / time.as_secs_f64(),
        );
    }

    if let Some((_, report)) = first {
        println!("\nAll four versions agree. The 5 paths that cost the most time:\n");
        print!("{}", report.render(5));
    }
}
