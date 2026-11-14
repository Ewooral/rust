//! Structured logging with `tracing`: events, fields, spans, and JSON output.
//! Run: cargo run -p capstone_rest_api --example structured_logging
//!      cargo run -p capstone_rest_api --example structured_logging -- json

use capstone_rest_api::config::LogFormat;
use capstone_rest_api::telemetry;
use tracing::{Level, info, instrument, span, warn};

/// `#[instrument]` opens a span for every call, recording the arguments as
/// fields (`skip` leaves out what shouldn't be logged, like a password).
#[instrument(skip(password))]
fn login(email: &str, password: &str) -> bool {
    let ok = password.len() >= 12;
    if ok {
        info!("password accepted");
    } else {
        warn!(length = password.len(), "password too short");
    }
    ok
}

fn main() {
    let json = std::env::args().nth(1).as_deref() == Some("json");
    let format = if json {
        LogFormat::Json
    } else {
        LogFormat::Pretty
    };
    telemetry::init_with(format, "info"); // this example's own events too

    // An event: a message plus named, typed fields (not a formatted string).
    info!(port = 8080, workers = 4, "server starting");

    // A span: everything logged inside it carries its fields too, here a
    // request id, exactly like the TraceLayer span around every request.
    let request = span!(Level::INFO, "request", request_id = 7, method = "POST");
    let _entered = request.enter();
    info!(path = "/api/v1/auth/login", "handling");
    login("ama@example.com", "correct horse battery");
    login("kofi@example.com", "short");
}
