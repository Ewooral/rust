//! Solution to exercise 1: make it compile (E0277), with fields you can search on.
//!
//! `place_order` logs the whole order as one field. tracing can only record
//! a few types directly as field values (numbers, bools, strings, errors...):
//! they implement the trait `tracing::Value`. A struct of yours doesn't, so
//! this doesn't compile.
//!
//! `?order` (record it with Debug) would compile, but then the order is one
//! opaque string, and nobody can filter the logs for "orders over £10".
//! The test wants the parts as separate, typed fields:
//! `order.id=7 order.total_pence=1250`. Write them like that.
//!
//! Test: cargo test -p observability --features solutions --bin sol01_fix_field_value

use tracing::info;

#[derive(Debug)]
struct Order {
    id: u32,
    total_pence: u64,
}

fn place_order(order: &Order) {
    // Separate, typed fields: searchable, and filterable by value.
    info!(
        order.id = order.id,
        order.total_pence = order.total_pence,
        "order placed"
    );
}

fn main() {
    tracing_subscriber::fmt().init();
    place_order(&Order {
        id: 7,
        total_pence: 1_250,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use observability::capture::CapturedLogs;

    #[test]
    fn the_order_is_logged_as_separate_fields() {
        let logs = CapturedLogs::default();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(logs.clone())
            .with_ansi(false)
            .without_time()
            .with_target(false)
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            place_order(&Order {
                id: 7,
                total_pence: 1_250,
            });
        });
        assert_eq!(
            logs.contents(),
            " INFO order placed order.id=7 order.total_pence=1250\n"
        );
    }
}
