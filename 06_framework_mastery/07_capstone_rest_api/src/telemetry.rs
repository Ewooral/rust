//! Structured logging with `tracing`.
//!
//! Log lines are EVENTS with named FIELDS (`user_id = 7`), inside SPANS
//! (one per request, carrying its `request_id`). The same events print as
//! readable text on your machine or as JSON lines in production, where a
//! log collector can search by field.

use std::io::IsTerminal;

use tracing_subscriber::EnvFilter;

use crate::config::LogFormat;

/// What to log when `RUST_LOG` isn't set: our crate and the request log at
/// `info`, and only warnings from everything else.
pub const DEFAULT_FILTER: &str = "warn,capstone_rest_api=info,tower_http=info";

/// Installs the global subscriber. Safe to call more than once (later calls
/// do nothing), which tests and examples rely on.
pub fn init(format: LogFormat) {
    init_with(format, DEFAULT_FILTER);
}

/// [`init`] with another default filter (`RUST_LOG` still wins).
pub fn init_with(format: LogFormat, default_filter: &str) {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));
    let builder = tracing_subscriber::fmt().with_env_filter(filter);
    let _ = match format {
        LogFormat::Pretty => builder
            .with_ansi(std::io::stdout().is_terminal())
            .try_init(),
        LogFormat::Json => builder.json().with_current_span(true).try_init(),
    };
}
