//! Mini project: a checkout service designed to be tested.
//!
//! The program wires up the PRODUCTION dependencies (the real clock and
//! unpredictable order ids) with a fake payment gateway, because this is a
//! lesson and we don't take real money. The tests swap every dependency.
//!
//! Run:  cargo run -p advanced_testing
//! Test: cargo test -p advanced_testing

use advanced_testing::checkout::{Cart, Checkout, Coupon};
use advanced_testing::clock::{Clock, SystemClock};
use advanced_testing::ids::OrderIds;
use advanced_testing::money::parse_money;
use advanced_testing::payment::{CardToken, FakeGateway};

fn main() {
    let clock = SystemClock;
    let week = 7 * 24 * 3_600;
    let mut checkout = Checkout::new(
        clock,
        FakeGateway::new().declining("tok_broke_0000"),
        OrderIds::from_os(),
    )
    .with_coupon(Coupon {
        code: "WELCOME20".to_string(),
        percent_off: 20,
        expires_at: clock.now().plus_seconds(week),
    });

    // Prices come from "outside" as text, so they go through the parser.
    let price = |text: &str| parse_money(text).expect("a valid price");
    let cart =
        Cart::new()
            .with("Flat white", price("£3.20"), 2)
            .with("Banana bread", price("2.75"), 1);

    let orders = [
        ("tok_visa_4242", None),
        ("tok_visa_4242", Some("welcome20")),
        ("tok_visa_4242", Some("SPRING99")),
        ("tok_broke_0000", None),
    ];
    for (card, coupon) in orders {
        println!("--- card {card}, coupon {coupon:?}");
        match checkout.place_order(&cart, &CardToken(card.to_string()), coupon) {
            Ok(receipt) => print!("{}", receipt.render()),
            Err(e) => println!("order refused: {e}"),
        }
    }
    println!(
        "--- the fake gateway took {} payments",
        checkout.payments().charges().len()
    );
}
