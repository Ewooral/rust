//! Property tests (recap of 03/02): rules that hold for EVERY input.

use advanced_testing::checkout::Cart;
use advanced_testing::clock::Timestamp;
use advanced_testing::money::{Money, parse_money};
use proptest::prelude::*;

use crate::support::{card, checkout_at};

// A custom strategy: any cart of 1 to 5 lines with sensible prices.
prop_compose! {
    fn any_cart()(lines in prop::collection::vec((1u64..100_000, 1u32..50), 1..6)) -> Cart {
        lines
            .into_iter()
            .enumerate()
            .fold(Cart::new(), |cart, (i, (pence, qty))| {
                cart.with(&format!("item {i}"), Money::from_pence(pence), qty)
            })
    }
}

// Any moment in 2026, as seconds since 1970.
fn any_time_in_2026() -> impl Strategy<Value = Timestamp> {
    let start = Timestamp::from_utc(2026, 1, 1, 0, 0).0;
    let end = Timestamp::from_utc(2027, 1, 1, 0, 0).0;
    (start..end).prop_map(Timestamp)
}

proptest! {
    // Fewer cases than the default 256: these run on every `cargo test`.
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn money_prints_and_parses_back(pence in any::<u64>()) {
        let money = Money::from_pence(pence);
        prop_assert_eq!(parse_money(&money.to_string()), Ok(money));
    }

    #[test]
    fn parsing_never_panics(text in ".{0,30}") {
        let _ = parse_money(&text);
    }

    #[test]
    fn totals_are_never_more_than_the_subtotal(cart in any_cart(), now in any_time_in_2026(), use_coupon: bool) {
        let mut checkout = checkout_at(now);
        let coupon = use_coupon.then_some("WELCOME20");
        // The coupon expires on 1 December, so some of these orders fail:
        // that's fine, the property is about the ones that succeed.
        if let Ok(receipt) = checkout.place_order(&cart, &card(), coupon) {
            prop_assert!(receipt.total <= receipt.subtotal);
            let discount = receipt.discount.map_or(Money::ZERO, |(_, amount)| amount);
            prop_assert_eq!(receipt.total.checked_add(discount), Some(receipt.subtotal));
            prop_assert_eq!(checkout.payments().charges()[0].amount, receipt.total);
        }
    }
}
