//! Snapshot tests: compare a whole rendered receipt with a saved copy.
//!
//! The saved copies are in `tests/checkout/snapshots/*.snap`. Read them:
//! they ARE the expected output. When the format changes on purpose, review
//! the new version (`cargo insta review`) instead of editing strings here.

use advanced_testing::checkout::{Cart, CheckoutError};
use advanced_testing::money::Money;
use insta::{assert_debug_snapshot, assert_snapshot};

use crate::support::{card, checkout_at, coffee_and_cake, monday_happy_hour, monday_morning};

#[test]
fn plain_receipt() {
    let mut checkout = checkout_at(monday_morning());
    let receipt = checkout
        .place_order(&coffee_and_cake(), &card(), None)
        .unwrap();
    assert_snapshot!(receipt.render());
}

#[test]
fn happy_hour_receipt() {
    let mut checkout = checkout_at(monday_happy_hour());
    let receipt = checkout
        .place_order(&coffee_and_cake(), &card(), None)
        .unwrap();
    assert_snapshot!(receipt.render());
}

#[test]
fn coupon_receipt() {
    let mut checkout = checkout_at(monday_happy_hour());
    let receipt = checkout
        .place_order(&coffee_and_cake(), &card(), Some("WELCOME20"))
        .unwrap();
    assert_snapshot!(receipt.render());
}

#[test]
fn receipt_as_debug_value() {
    // assert_debug_snapshot! saves the {:#?} form: handy for whole structs.
    let mut checkout = checkout_at(monday_morning());
    let cart = Cart::new().with("Espresso", Money::from_pence(250), 1);
    let receipt = checkout.place_order(&cart, &card(), None).unwrap();
    assert_debug_snapshot!(receipt);
}

#[test]
fn error_messages() {
    // An INLINE snapshot: the expected value lives right here in the source.
    let mut checkout = checkout_at(monday_morning());
    let messages: Vec<String> = [
        checkout.place_order(&Cart::new(), &card(), None),
        checkout.place_order(&coffee_and_cake(), &card(), Some("SPRING99")),
    ]
    .into_iter()
    .map(|r| r.map_or_else(|e: CheckoutError| e.to_string(), |_| "ok".to_string()))
    .collect();
    assert_snapshot!(messages.join("\n"), @r"
    the cart is empty
    no such coupon: SPRING99
    ");
}
