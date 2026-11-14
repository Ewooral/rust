//! End-to-end flows through the public API, with fakes and a mock.

use advanced_testing::checkout::{Checkout, CheckoutError};
use advanced_testing::clock::FixedClock;
use advanced_testing::ids::OrderIds;
use advanced_testing::money::Money;
use advanced_testing::payment::{CardToken, PaymentError, PaymentId};
use mockall::Sequence;

use crate::support::{
    MockGateway, card, checkout_at, coffee_and_cake, monday_happy_hour, monday_morning,
};

#[test]
fn a_successful_order_charges_the_total_with_the_order_id_as_key() {
    let mut checkout = checkout_at(monday_morning());

    let receipt = checkout
        .place_order(&coffee_and_cake(), &card(), None)
        .unwrap();

    // The fake gateway is also a spy: ask it what happened.
    let charges = checkout.payments().charges();
    assert_eq!(charges.len(), 1);
    assert_eq!(charges[0].amount, receipt.total);
    assert_eq!(charges[0].idempotency_key, receipt.order_id);
}

#[test]
fn the_same_seed_gives_the_same_order_ids() {
    let mut first = checkout_at(monday_morning());
    let mut second = checkout_at(monday_morning());
    for _ in 0..3 {
        let a = first
            .place_order(&coffee_and_cake(), &card(), None)
            .unwrap();
        let b = second
            .place_order(&coffee_and_cake(), &card(), None)
            .unwrap();
        assert_eq!(a.order_id, b.order_id);
    }
}

#[test]
fn coupon_codes_are_case_insensitive() {
    let mut checkout = checkout_at(monday_morning());

    let receipt = checkout
        .place_order(&coffee_and_cake(), &card(), Some("welcome20"))
        .unwrap();

    assert_eq!(receipt.total, Money::from_pence(732));
}

#[test]
fn an_unreachable_provider_is_reported_and_happy_hour_was_applied_first() {
    let mut gateway = MockGateway::new();
    gateway
        .expect_charge()
        .withf(|_, amount, _| *amount == Money::from_pence(824))
        .times(1)
        .returning(|_, _, _| Err(PaymentError::Unavailable));
    let mut checkout = Checkout::new(
        FixedClock::at(monday_happy_hour()),
        gateway,
        OrderIds::seeded(1),
    );

    let result = checkout.place_order(&coffee_and_cake(), &card(), None);

    assert_eq!(
        result,
        Err(CheckoutError::Payment(PaymentError::Unavailable))
    );
}

#[test]
fn two_customers_are_charged_in_order() {
    // A Sequence makes mockall check the ORDER of calls, not just how many.
    let mut seq = Sequence::new();
    let mut gateway = MockGateway::new();
    for (card, id) in [("tok_ada_1111", "pay_a"), ("tok_bob_2222", "pay_b")] {
        gateway
            .expect_charge()
            .withf(move |c, _, _| c.0 == card)
            .times(1)
            .in_sequence(&mut seq)
            .returning(move |_, _, _| Ok(PaymentId(id.to_string())));
    }
    let mut checkout = Checkout::new(
        FixedClock::at(monday_morning()),
        gateway,
        OrderIds::seeded(1),
    );

    let ada = checkout.place_order(&coffee_and_cake(), &CardToken("tok_ada_1111".into()), None);
    let bob = checkout.place_order(&coffee_and_cake(), &CardToken("tok_bob_2222".into()), None);

    assert_eq!(ada.unwrap().payment_id.0, "pay_a");
    assert_eq!(bob.unwrap().payment_id.0, "pay_b");
}
