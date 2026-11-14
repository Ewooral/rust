//! A fuzz harness that also runs as an ordinary test.
//!
//! A fuzzer feeds a function millions of byte strings, keeps the ones that
//! reach new code, and mutates those. `cargo fuzz` (libFuzzer, nightly Rust)
//! does it for real; see the README. The `arbitrary` crate turns raw bytes
//! into structured values, so the SAME harness function works under
//! cargo-fuzz and here, where a seeded generator supplies the bytes.

use advanced_testing::checkout::{Cart, CheckoutError};
use advanced_testing::clock::Timestamp;
use advanced_testing::money::{Money, check_parse_money};
use arbitrary::{Arbitrary, Unstructured};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::support::{card, checkout_at};

/// A whole order, built from fuzzer bytes.
#[derive(Debug, Arbitrary)]
struct FuzzOrder {
    lines: Vec<(u64, u32)>,
    seconds_since_2026: u32,
    coupon: Option<String>,
}

/// Harness 1, for the money parser: `fuzz_target!(|text: &str| ...)`.
/// libfuzzer-sys builds a typed input with `arbitrary_take_rest`, which
/// uses ALL the bytes, so a seed like b"12.34" really is the text "12.34".
fn fuzz_money(data: &[u8]) {
    if let Ok(text) = <&str>::arbitrary_take_rest(Unstructured::new(data)) {
        check_parse_money(text);
    }
}

/// Harness 2, for the checkout: `fuzz_target!(|order: FuzzOrder| ...)`.
/// Must not panic for ANY bytes; if it does, the fuzzer has found a bug.
fn fuzz_order(data: &[u8]) {
    let Ok(order) = FuzzOrder::arbitrary_take_rest(Unstructured::new(data)) else {
        return; // not enough bytes: nothing to test
    };
    let cart = order.lines.iter().fold(Cart::new(), |cart, &(pence, qty)| {
        cart.with("x", Money::from_pence(pence), qty)
    });
    let now =
        Timestamp(Timestamp::from_utc(2026, 1, 1, 0, 0).0 + u64::from(order.seconds_since_2026));
    let mut checkout = checkout_at(now);
    match checkout.place_order(&cart, &card(), order.coupon.as_deref()) {
        Ok(receipt) => {
            assert!(receipt.total <= receipt.subtotal, "{order:?}");
            assert_eq!(checkout.payments().charges().len(), 1, "{order:?}");
        }
        Err(CheckoutError::Payment(e)) => panic!("the fake gateway never fails: {e}"),
        Err(_) => assert!(
            checkout.payments().charges().is_empty(),
            "charged for a refused order: {order:?}"
        ),
    }
}

fn fuzz_one(data: &[u8]) {
    fuzz_money(data);
    fuzz_order(data);
}

/// Inputs a human thought were interesting (a fuzzer calls this the corpus).
const SEEDS: [&[u8]; 6] = [
    b"",
    b"\x00",
    b"12.34",
    "£1.00".as_bytes(),
    b"184467440737095516.16",
    b"\xff\xff\xff\xff\xff",
];

#[test]
fn harness_survives_the_seed_corpus() {
    for seed in SEEDS {
        fuzz_one(seed);
    }
}

#[test]
fn harness_survives_ten_thousand_random_inputs() {
    // Seeded, so a failure here fails the same way on every run.
    let mut rng = StdRng::seed_from_u64(0xF022);
    let mut buffer = Vec::with_capacity(256);
    for _ in 0..10_000 {
        buffer.clear();
        let len = rng.random_range(0..256);
        buffer.extend((0..len).map(|_| rng.random::<u8>()));
        fuzz_one(&buffer);
    }
}
