//! Lesson 05/06: observability.
//!
//! The mini project is an instrumented job processor: spans for every job
//! and attempt, events at levels chosen for the reader, text or JSON output
//! filtered by `RUST_LOG`, and metrics computed from the same events.
//!
//! - [`processor`]: the instrumented processor (spans, levels, async tasks).
//! - [`jobs`]: the jobs and their (deterministic) failures.
//! - [`telemetry`]: building the subscriber: layers, filter, format.
//! - [`metrics`]: a custom `Layer` that turns events into numbers.
//! - [`capture`]: an in-memory writer, so tests can assert on log output.

pub mod capture;
pub mod jobs;
pub mod metrics;
pub mod processor;
pub mod telemetry;
