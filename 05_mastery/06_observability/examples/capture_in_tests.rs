//! Testing log output: capture it in memory and assert on it.
//!
//! Run:  cargo run -p observability --example capture_in_tests
//! Test: cargo test -p observability --example capture_in_tests
//! See with_test_writer: cargo test -p observability --example capture_in_tests -- --show-output

use tracing::{info, warn};

/// The code under test: logging is part of its behaviour. An operator
/// relies on that warning to approve large transfers.
fn transfer(from: &str, to: &str, pence: u64) -> Result<(), String> {
    if pence > 100_000 {
        warn!(from, to, pence, "large transfer held for approval");
        return Err("needs approval".to_string());
    }
    info!(from, to, pence, "transfer done");
    Ok(())
}

fn main() {
    tracing_subscriber::fmt().without_time().init();
    let _ = transfer("ama", "kofi", 2_500);
    let _ = transfer("ama", "kofi", 250_000);
}

#[cfg(test)]
mod tests {
    use super::*;
    use observability::capture::CapturedLogs;
    use tracing::subscriber::DefaultGuard;

    /// Text output into memory. `set_default` lasts until the guard is
    /// dropped and only affects THIS thread, so parallel tests can't see
    /// each other's logs.
    fn capture_text() -> (CapturedLogs, DefaultGuard) {
        let logs = CapturedLogs::default();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(logs.clone())
            .with_ansi(false)
            .without_time()
            .with_target(false)
            .finish();
        (logs, tracing::subscriber::set_default(subscriber))
    }

    #[test]
    fn large_transfers_log_a_warning() {
        let (logs, _guard) = capture_text();

        let _ = transfer("ama", "kofi", 250_000);

        assert_eq!(
            logs.contents(),
            " WARN large transfer held for approval from=\"ama\" to=\"kofi\" pence=250000\n"
        );
    }

    #[test]
    fn json_makes_assertions_robust() {
        // Comparing whole text lines breaks when the format changes. With
        // JSON you parse the line and check only the fields you care about.
        let logs = CapturedLogs::default();
        let subscriber = tracing_subscriber::fmt()
            .json()
            .with_writer(logs.clone())
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            transfer("ama", "kofi", 2_500).unwrap();
        });

        let event: serde_json::Value = serde_json::from_str(&logs.lines()[0]).unwrap();
        assert_eq!(event["level"], "INFO");
        assert_eq!(event["fields"]["pence"], 2_500);
        assert_eq!(event["fields"]["message"], "transfer done");
    }

    #[test]
    fn with_test_writer_only_shows_logs_when_asked() {
        // `with_test_writer()` doesn't let you assert: it prints through the
        // test harness, which hides the output of passing tests. Use it to
        // SEE logs: they appear for failing tests, or with --show-output.
        let subscriber = tracing_subscriber::fmt()
            .with_test_writer()
            .without_time()
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            transfer("ama", "kofi", 1).unwrap();
        });
    }
}
