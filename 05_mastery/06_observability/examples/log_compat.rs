//! The `log` crate: many libraries still log with it. A tracing subscriber
//! can collect those records too.
//!
//! Run: cargo run -p observability --example log_compat

/// Stands for a dependency that uses `log`, not `tracing`.
mod legacy_library {
    pub fn connect(host: &str) {
        log::info!("connecting to {host}");
        log::debug!("using TLS 1.3");
        log::warn!(target: "legacy::pool", "connection pool almost full");
    }
}

fn main() {
    // `init()` sets the global subscriber AND, because tracing-subscriber's
    // `tracing-log` feature is on by default, installs a `LogTracer`: it
    // turns every `log` record into a tracing event.
    tracing_subscriber::fmt()
        .without_time()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    tracing::info!(host = "db.local", "a tracing event, with a field");
    legacy_library::connect("db.local");
    tracing::info!("both kinds arrive in the same place, in order");
}
