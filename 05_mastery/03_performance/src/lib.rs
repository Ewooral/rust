//! Lesson 05/03: performance.
//!
//! The mini project is a log-analysis pipeline written four times, each
//! version faster than the last, with benchmarks (`benches/`) to prove it.
//!
//! - [`pipeline`]: the four versions of the log report.
//! - [`logs`]: a seeded generator of realistic access logs.
//! - [`alloc_counter`]: a global allocator that counts allocations.
//! - [`kernels`]: small loops for the data-layout, iterator and
//!   bounds-check comparisons.
//! - [`timing`]: a median-of-N stopwatch for programs and examples.
//!
//! Run `cargo run -p performance --release` and `cargo bench -p performance`.

pub mod alloc_counter;
pub mod kernels;
pub mod logs;
pub mod pipeline;
pub mod timing;

// The unit tests count allocations, so the test build of this library
// installs the counting allocator. (A program can have only one global
// allocator: `src/main.rs` and each example install their own.)
#[cfg(test)]
#[global_allocator]
static GLOBAL: alloc_counter::CountingAlloc = alloc_counter::CountingAlloc;
