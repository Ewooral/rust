//! Fields: structured key=value data on every event, not just a sentence.
//!
//! Run: cargo run -p observability --example fields_and_values

use std::fmt;

use tracing::{field, info, info_span};

#[derive(Debug)]
struct Order {
    id: u32,
    items: Vec<&'static str>,
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "order #{} ({} items)", self.id, self.items.len())
    }
}

fn main() {
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .init();

    // A field is `name = value`. Numbers, bools and strings stay typed.
    info!(user_id = 42, admin = false, "user logged in");

    // Dotted names group related fields.
    info!(user.name = "ama", user.country = "GH", "profile loaded");

    // A variable on its own: its name becomes the field name.
    let path = "/api/orders";
    info!(path, "request received");

    // Your own types: `?` records them with Debug, `%` with Display...
    let order = Order {
        id: 7,
        items: vec!["tea", "scone"],
    };
    info!(?order, "with ? (Debug)");
    info!(%order, "with % (Display)");
    // ...or record the parts you'll want to search and filter on.
    info!(
        order.id = order.id,
        order.items = order.items.len(),
        "chosen fields"
    );

    // The message can use format arguments, like println!.
    info!(attempt = 2, "retrying in {} seconds", 5);

    // A span field can be declared Empty and filled in later.
    let span = info_span!("request", path, status = field::Empty);
    let _entered = span.enter();
    info!("handling");
    span.record("status", 200);
    info!("handled");
}
