//! Building the subscriber: which layers, which filter, which format.
//!
//! A program sets this up ONCE, at the start of `main`. Libraries never do:
//! they only emit spans and events, and the application decides where they
//! go. That's the split that lets one subscriber collect logs from every
//! crate in the program.

use tracing::Subscriber;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, fmt};

use crate::metrics::Metrics;

/// How log lines look.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Human-readable lines, for a terminal.
    Text {
        /// Colour codes (only when writing to a real terminal).
        ansi: bool,
    },
    /// One JSON object per line, for log collectors.
    Json,
}

/// A registry with three layers:
/// 1. text OR json output (only one is `Some`), filtered by `filter`;
/// 2. the metrics layer, which is NOT filtered, so its numbers count every
///    event even when the output is quiet. That's a per-layer filter
///    (`with_filter`), as opposed to a global one (`.with(filter)`).
///
/// `writer` decides where lines go: `std::io::stderr` in `main`, a
/// [`CapturedLogs`](crate::capture::CapturedLogs) in tests.
pub fn subscriber<W>(
    format: Format,
    filter: EnvFilter,
    writer: W,
    metrics: &Metrics,
) -> impl Subscriber + Send + Sync
where
    W: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let (text, json) = match format {
        Format::Text { ansi } => (
            Some(
                fmt::layer()
                    .with_writer(writer)
                    .with_ansi(ansi)
                    .with_filter(filter),
            ),
            None,
        ),
        Format::Json => (
            None,
            Some(
                fmt::layer()
                    .json()
                    .with_current_span(true) // the innermost span's fields
                    .with_span_list(false) // not every enclosing span
                    .with_writer(writer)
                    .with_filter(filter),
            ),
        ),
    };
    // `Option<Layer>` is a layer too: `None` simply does nothing.
    tracing_subscriber::registry()
        .with(text)
        .with(json)
        .with(metrics.layer())
}

/// The filter `main` uses: `RUST_LOG` if it is set (and valid), else `info`.
pub fn filter_from_env() -> EnvFilter {
    EnvFilter::builder()
        .with_default_directive(tracing::level_filters::LevelFilter::INFO.into())
        .from_env_lossy()
}
