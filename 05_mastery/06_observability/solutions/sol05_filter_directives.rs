//! Solution to exercise 5: write the `RUST_LOG` directives.
//!
//! `workload` logs from two modules, `database` and `http`, and from inside
//! a `checkout` span. Each function below returns the directive string an
//! operator would put in `RUST_LOG` to see exactly what the test expects.
//! An event's target is normally its module path; here the two modules
//! set short targets of their own (`target: "database"`). Syntax:
//!
//!   warn                 a level for everything
//!   database=debug       a level for one target
//!   http=off             silence one target
//!   [checkout]=debug     events inside `checkout` spans
//!   info,database=debug  combine with commas
//!
//! Replace each `todo!()` with a string literal.
//!
//! Test: cargo test -p observability --features solutions --bin sol05_filter_directives

use tracing::{debug, info, info_span, warn};

mod database {
    pub fn query() {
        tracing::debug!(target: "database", "SELECT 1");
        tracing::info!(target: "database", "query done");
    }
}

mod http {
    pub fn request() {
        tracing::info!(target: "http", "GET /");
        tracing::warn!(target: "http", "slow request");
    }
}

fn workload() {
    info!("start");
    database::query();
    http::request();
    let _span = info_span!("checkout").entered();
    debug!("checking stock");
    warn!("stock is low");
}

/// Only warnings and errors, from everywhere.
fn only_problems() -> &'static str {
    "warn"
}

/// The usual INFO, plus the database's DEBUG lines.
fn info_and_database_debug() -> &'static str {
    "info,database=debug"
}

/// INFO, but nothing at all from `http`.
fn info_without_http() -> &'static str {
    "info,http=off"
}

/// Only warnings, except: everything down to DEBUG inside `checkout` spans.
fn warnings_and_checkout_debug() -> &'static str {
    "warn,[checkout]=debug"
}

fn main() {
    for (name, directives) in [
        ("only_problems", only_problems()),
        ("info_and_database_debug", info_and_database_debug()),
        ("info_without_http", info_without_http()),
        ("warnings_and_checkout_debug", warnings_and_checkout_debug()),
    ] {
        println!("--- {name}: RUST_LOG={directives:?}");
        let subscriber = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::new(directives))
            .without_time()
            .finish();
        tracing::subscriber::with_default(subscriber, workload);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use observability::capture::CapturedLogs;
    use tracing_subscriber::EnvFilter;

    /// Runs `workload` with these directives; returns the messages logged.
    fn messages(directives: &str) -> Vec<String> {
        let logs = CapturedLogs::default();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(logs.clone())
            .with_env_filter(EnvFilter::new(directives))
            .with_ansi(false)
            .without_time()
            .with_level(false)
            .with_target(false)
            .finish();
        tracing::subscriber::with_default(subscriber, workload);
        logs.lines()
            .iter()
            .map(|line| line.rsplit(": ").next().unwrap().trim().to_string())
            .collect()
    }

    #[test]
    fn only_problems_shows_warnings() {
        assert_eq!(messages(only_problems()), ["slow request", "stock is low"]);
    }

    #[test]
    fn database_debug_is_added() {
        assert_eq!(
            messages(info_and_database_debug()),
            [
                "start",
                "SELECT 1",
                "query done",
                "GET /",
                "slow request",
                "stock is low"
            ]
        );
    }

    #[test]
    fn http_is_silenced() {
        assert_eq!(
            messages(info_without_http()),
            ["start", "query done", "stock is low"]
        );
    }

    #[test]
    fn checkout_gets_debug() {
        assert_eq!(
            messages(warnings_and_checkout_debug()),
            ["slow request", "checking stock", "stock is low"]
        );
    }
}
