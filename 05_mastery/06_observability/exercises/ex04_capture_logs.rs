//! Exercise 4: build the log-capturing writer the tests use.
//!
//! To assert on log output, a test points the `fmt` subscriber at a writer
//! that keeps everything in memory. The subscriber calls `make_writer` once
//! per log line, so `MemoryLogs` must hand out writers that all append to
//! ONE shared buffer, and the test must be able to read that buffer later.
//!
//! Implement the four `todo!()`s:
//! - `MemoryLogs` holds an `Arc<Mutex<Vec<u8>>>` (already declared);
//! - `text()` returns everything written so far as a `String`;
//! - `io::Write::write` appends the bytes and returns how many it took;
//! - `make_writer` returns a writer that shares the same buffer (a clone).
//!
//! Test: cargo test -p observability --features exercises --bin ex04_capture_logs

#![allow(unused_variables, dead_code)]

use std::io;
use std::sync::{Arc, Mutex};

use tracing_subscriber::fmt::MakeWriter;

#[derive(Debug, Clone, Default)]
struct MemoryLogs(Arc<Mutex<Vec<u8>>>);

impl MemoryLogs {
    fn text(&self) -> String {
        todo!()
    }
}

impl io::Write for MemoryLogs {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        todo!()
    }

    fn flush(&mut self) -> io::Result<()> {
        todo!()
    }
}

impl<'a> MakeWriter<'a> for MemoryLogs {
    type Writer = MemoryLogs;

    fn make_writer(&'a self) -> Self::Writer {
        todo!()
    }
}

fn main() {
    let logs = MemoryLogs::default();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(logs.clone())
        .with_ansi(false)
        .finish();
    tracing::subscriber::with_default(subscriber, || tracing::info!("hello"));
    print!("captured: {}", logs.text());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capture(f: impl FnOnce()) -> String {
        let logs = MemoryLogs::default();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(logs.clone())
            .with_ansi(false)
            .without_time()
            .with_target(false)
            .finish();
        tracing::subscriber::with_default(subscriber, f);
        logs.text()
    }

    #[test]
    fn captures_one_line() {
        assert_eq!(
            capture(|| tracing::warn!(disk = "C:", "low space")),
            " WARN low space disk=\"C:\"\n"
        );
    }

    #[test]
    fn captures_lines_in_order() {
        let text = capture(|| {
            tracing::info!("one");
            tracing::info!("two");
        });
        assert_eq!(text, " INFO one\n INFO two\n");
    }

    #[test]
    fn captures_nothing_below_the_level() {
        assert_eq!(capture(|| tracing::debug!("hidden")), "");
    }
}
