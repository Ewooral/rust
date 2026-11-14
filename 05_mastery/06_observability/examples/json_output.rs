//! JSON output: one object per line, for machines (log collectors, jq).
//!
//! Run: cargo run -p observability --example json_output

use tracing::{info, info_span};

fn work() {
    let request = info_span!("request", method = "GET", path = "/orders");
    let _r = request.enter();
    let db = info_span!("db_query", table = "orders");
    let _d = db.enter();
    info!(rows = 3, "query finished");
}

fn main() {
    println!("--- .json(): fields nested, current span AND the list of all spans");
    let subscriber = tracing_subscriber::fmt().json().without_time().finish();
    tracing::subscriber::with_default(subscriber, work);

    println!("--- .json().flatten_event(true).with_span_list(false)");
    let subscriber = tracing_subscriber::fmt()
        .json()
        .flatten_event(true) // event fields at the top level, not under "fields"
        .with_span_list(false) // only the innermost span
        .without_time()
        .finish();
    tracing::subscriber::with_default(subscriber, work);

    println!("--- the same event as text, for comparison");
    let subscriber = tracing_subscriber::fmt().without_time().finish();
    tracing::subscriber::with_default(subscriber, work);
}
