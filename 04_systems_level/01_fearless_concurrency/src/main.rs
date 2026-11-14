//! Mini project for Book chapter 16: count the words in 2,000 documents
//! five different ways, check that every way gives the same answer, and
//! time them. The strategies live in `src/lib.rs`.
//!
//!   cargo run -p fearless_concurrency               one worker per CPU core
//!   cargo run -p fearless_concurrency -- 2          exactly 2 workers
//!   cargo run -p fearless_concurrency --release     see the real speed-up
//!   cargo test -p fearless_concurrency

use fearless_concurrency::{
    WordCounts, count_sequential, count_with_rayon, count_with_scoped_threads,
    count_with_shared_map, count_with_worker_pool, sample_corpus, top_words,
};
use std::env;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    let workers = env::args()
        .nth(1)
        .and_then(|arg| arg.parse().ok())
        .unwrap_or_else(|| thread::available_parallelism().map_or(4, |n| n.get()));

    let corpus = sample_corpus(2_000, 200);
    println!(
        "Counting {} words in {} documents with {workers} workers\n",
        corpus.len() * 200,
        corpus.len()
    );

    let (expected, seq_time) = timed(|| count_sequential(&corpus));
    let ((pooled, reports), pool_time) = timed(|| count_with_worker_pool(corpus.clone(), workers));
    let shared_docs: Arc<[String]> = corpus.clone().into();
    let (shared, shared_time) = timed(|| count_with_shared_map(shared_docs, workers));
    let (scoped, scoped_time) = timed(|| count_with_scoped_threads(&corpus, workers));
    let (rayon, rayon_time) = timed(|| count_with_rayon(&corpus));

    println!("| strategy                              | time       | same answer? |");
    println!("|---------------------------------------|------------|--------------|");
    row("1. sequential (baseline)", seq_time, &expected, &expected);
    row(
        "2. worker pool: channels + Arc<Mutex>",
        pool_time,
        &pooled,
        &expected,
    );
    row(
        "3. spawn + Arc<Mutex<HashMap>>",
        shared_time,
        &shared,
        &expected,
    );
    row("4. thread::scope + chunks", scoped_time, &scoped, &expected);
    row("5. rayon par_iter + reduce", rayon_time, &rayon, &expected);

    println!("\nHow the worker pool shared out the documents:");
    for report in &reports {
        println!(
            "  worker {:>2}: {:>4} documents",
            report.worker, report.docs_counted
        );
    }

    println!("\nTop 10 words of {} different words:", expected.len());
    for (rank, (word, count)) in top_words(&expected, 10).into_iter().enumerate() {
        println!("  {:>2}. {word:<10} {count}", rank + 1);
    }
    println!("\n(Debug builds are slow; try `--release` to compare the timings.)");
}

/// Runs `job` and measures how long it took.
fn timed<T>(job: impl FnOnce() -> T) -> (T, Duration) {
    let start = Instant::now();
    let value = job();
    (value, start.elapsed())
}

fn row(name: &str, time: Duration, got: &WordCounts, expected: &WordCounts) {
    let same = if got == expected { "yes" } else { "NO!" };
    println!(
        "| {name:<37} | {:>7.1} ms | {same:<12} |",
        time.as_secs_f64() * 1000.0
    );
}
