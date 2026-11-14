//! EnvFilter: choose what to see with directives, usually from `RUST_LOG`.
//!
//! Run: cargo run -p observability --example env_filter
//! Then with your own directives (they apply to the last section):
//!   bash:        RUST_LOG="env_filter::database=trace" cargo run -p observability --example env_filter
//!   PowerShell:  $env:RUST_LOG="env_filter::database=trace"; cargo run -p observability --example env_filter

use tracing::{debug, info, info_span, trace, warn};
use tracing_subscriber::EnvFilter;

// An event's TARGET is its module path, e.g. `env_filter::database`
// (`env_filter` is the name of this example's crate).
mod database {
    pub fn query() {
        tracing::trace!("SELECT * FROM orders");
        tracing::debug!(rows = 3, "query finished");
    }
}

mod http {
    pub fn request() {
        tracing::debug!("GET /api/orders");
        tracing::warn!(ms = 950, "slow request");
    }
}

fn workload() {
    info!("start");
    database::query();
    http::request();
    let _checkout = info_span!("checkout", order = 7).entered();
    debug!("inside the checkout span");
    trace!("very detailed checkout step");
    warn!("stock is low");
}

fn run_with(filter: EnvFilter, label: &str) {
    println!("--- {label}");
    let subscriber = tracing_subscriber::fmt()
        .without_time()
        .with_env_filter(filter)
        .finish();
    tracing::subscriber::with_default(subscriber, workload);
}

fn main() {
    for directives in [
        "info",                                     // INFO and more severe, everywhere
        "warn",                                     // only WARN and ERROR
        "info,env_filter::database=debug",          // info, but DEBUG for one module
        "debug,env_filter::http=off",               // debug, but silence one module
        "warn,[checkout]=trace",                    // everything inside `checkout` spans
        "warn,env_filter[checkout{order=7}]=debug", // ...only where order=7
    ] {
        run_with(
            EnvFilter::new(directives),
            &format!("EnvFilter::new({directives:?})"),
        );
    }

    // What a program normally does: RUST_LOG if it is set, else a default.
    let from_env = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    run_with(from_env, "RUST_LOG (or \"info\" when it is not set)");
}
