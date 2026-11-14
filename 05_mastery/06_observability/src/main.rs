//! Mini project: an instrumented job processor.
//!
//! Logs go to stderr; the final summary goes to stdout.
//!
//! Run:  cargo run -p observability
//!       cargo run -p observability -- --json
//! Choose how much with RUST_LOG (for this one command only):
//!       bash:        RUST_LOG=debug cargo run -p observability
//!       PowerShell:  $env:RUST_LOG="debug"; cargo run -p observability
//!                    (and Remove-Item Env:RUST_LOG afterwards)
//! Test: cargo test -p observability

use std::io::IsTerminal;

use observability::jobs::sample_jobs;
use observability::metrics::Metrics;
use observability::processor::JobProcessor;
use observability::telemetry::{self, Format};
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    let format = if std::env::args().any(|arg| arg == "--json") {
        Format::Json
    } else {
        Format::Text {
            ansi: std::io::stderr().is_terminal(),
        }
    };

    // Set up the subscriber once, first thing. `init` makes it the global
    // default for every thread, and also forwards `log` records to it.
    let metrics = Metrics::default();
    telemetry::subscriber(
        format,
        telemetry::filter_from_env(),
        std::io::stderr,
        &metrics,
    )
    .init();

    let report = JobProcessor::new(3).run_batch(sample_jobs()).await;

    let m = metrics.snapshot();
    println!("succeeded {}, failed {}", report.succeeded, report.failed);
    println!("events by level: {:?}", m.events);
    println!("outcomes: {:?}", m.outcomes);
    println!(
        "{} job spans closed, {:.0?} of job time in total",
        m.jobs_finished, m.job_time
    );
}
