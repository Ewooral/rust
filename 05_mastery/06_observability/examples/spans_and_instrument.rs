//! Spans: a period of time with a name and fields. `#[instrument]`: a span
//! for a whole function.
//!
//! Run: cargo run -p observability --example spans_and_instrument

use tracing::{Level, debug, info, info_span, instrument};
use tracing_subscriber::fmt::format::FmtSpan;

struct User {
    name: String,
}

/// A span named `add_to_cart` with the fields `user`, `item`, `quantity`
/// (every argument, recorded with Debug).
#[instrument]
fn add_to_cart(user: &str, item: &str, quantity: u32) -> u32 {
    info!("added");
    quantity * 250
}

/// `skip(password)`: never record it. `fields(...)`: add our own fields.
/// `ret`: log the return value. `err`: log an `Err` as an ERROR event.
#[instrument(skip(user, password), fields(user = %user.name), ret, err)]
fn log_in(user: &User, password: &str) -> Result<u32, String> {
    if password.len() >= 8 {
        Ok(42)
    } else {
        Err("password too short".to_string())
    }
}

/// Spans have levels too: this one only exists when DEBUG is enabled.
#[instrument(level = "debug")]
fn price_with_tax(pence: u32) -> u32 {
    debug!("adding 20% VAT");
    pence * 120 / 100
}

fn main() {
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_max_level(Level::DEBUG)
        // Also log when each span closes, with how long it was busy/idle.
        .with_span_events(FmtSpan::CLOSE)
        .init();

    // A span by hand: create it, then enter it. While entered, every event
    // is "inside" it, and the output shows its name and fields.
    let checkout = info_span!("checkout", order_id = 7);
    let entered = checkout.enter();
    info!("started");
    let subtotal = add_to_cart("ama", "tea", 2); // a child span of `checkout`
    let total = price_with_tax(subtotal);
    info!(total, "priced");
    drop(entered); // leave the span (dropping the guard exits it)

    // in_scope: enter, run the closure, exit.
    checkout.in_scope(|| info!("back inside for a moment"));

    let ama = User {
        name: "ama".to_string(),
    };
    let _ = log_in(&ama, "correct horse battery");
    let _ = log_in(&ama, "123");
}
