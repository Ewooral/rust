//! Test design: arrange/act/assert, names that explain, table-driven cases.
//!
//! Run:  cargo run -p advanced_testing --example test_design
//! Test: cargo test -p advanced_testing --example test_design

/// Shipping cost in pence. Orders of MORE than £50 ship free. Otherwise it
/// goes by weight: up to 1 kg £3.50, up to 5 kg £6.00, heavier £12.00.
/// Express delivery doubles the price (and is never free).
fn shipping_pence(order_pence: u64, weight_grams: u32, express: bool) -> u64 {
    if order_pence > 5_000 && !express {
        return 0;
    }
    let base = match weight_grams {
        0..=1_000 => 350,
        1_001..=5_000 => 600,
        _ => 1_200,
    };
    if express { base * 2 } else { base }
}

fn main() {
    for (order, grams, express) in [
        (2_000, 500, false),
        (2_000, 500, true),
        (6_000, 9_000, false),
    ] {
        println!(
            "order {order:>5}p, {grams:>5} g, express {express:<5} -> shipping {}p",
            shipping_pence(order, grams, express)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A good name says WHAT behaviour, and WHEN. In the test output it reads
    // like a line of the specification:
    //   test tests::orders_over_fifty_pounds_ship_free ... ok
    #[test]
    fn orders_over_fifty_pounds_ship_free() {
        // Arrange: set up exactly what this behaviour needs, nothing else.
        let order = 5_001;
        let heavy = 9_000;

        // Act: one call to the thing being tested.
        let cost = shipping_pence(order, heavy, false);

        // Assert: check the one outcome this test is about.
        assert_eq!(cost, 0);
    }

    // Bugs live at the edges: test both sides of every boundary.
    #[test]
    fn exactly_fifty_pounds_is_not_free() {
        assert_eq!(shipping_pence(5_000, 500, false), 350);
    }

    #[test]
    fn express_is_never_free_and_costs_double() {
        assert_eq!(shipping_pence(9_999, 500, true), 700);
    }

    // Table-driven: many cases, one loop. Each case has a name, and the
    // assertion message says which case failed.
    #[test]
    fn weight_bands() {
        let cases = [
            ("empty box", 0, 350),
            ("exactly 1 kg", 1_000, 350),
            ("just over 1 kg", 1_001, 600),
            ("exactly 5 kg", 5_000, 600),
            ("just over 5 kg", 5_001, 1_200),
        ];
        for (name, grams, expected) in cases {
            assert_eq!(
                shipping_pence(1_000, grams, false),
                expected,
                "case: {name}"
            );
        }
    }
}
