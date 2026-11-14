//! Shared test helpers: fixtures with meaningful names, and a mock.
//!
//! Good fixtures say what MATTERS about the data ("monday_happy_hour"),
//! so the tests that use them read like the business rules.

use advanced_testing::checkout::{Cart, Checkout, Coupon};
use advanced_testing::clock::{FixedClock, Timestamp};
use advanced_testing::ids::OrderIds;
use advanced_testing::money::Money;
use advanced_testing::payment::{CardToken, FakeGateway, PaymentError, PaymentGateway, PaymentId};

pub fn monday_morning() -> Timestamp {
    Timestamp::from_utc(2026, 11, 16, 9, 0)
}

pub fn monday_happy_hour() -> Timestamp {
    Timestamp::from_utc(2026, 11, 16, 15, 30)
}

pub fn card() -> CardToken {
    CardToken("tok_visa_4242".to_string())
}

pub fn coffee_and_cake() -> Cart {
    Cart::new()
        .with("Flat white", Money::from_pence(320), 2)
        .with("Banana bread", Money::from_pence(275), 1)
}

pub fn welcome_coupon() -> Coupon {
    Coupon {
        code: "WELCOME20".to_string(),
        percent_off: 20,
        expires_at: Timestamp::from_utc(2026, 12, 1, 0, 0),
    }
}

/// A checkout with a fake gateway, a fixed clock and seeded ids: every run
/// gives exactly the same receipts.
pub fn checkout_at(now: Timestamp) -> Checkout<FixedClock, FakeGateway> {
    Checkout::new(
        FixedClock::at(now),
        FakeGateway::new(),
        OrderIds::seeded(2026),
    )
    .with_coupon(welcome_coupon())
}

// `#[automock]` mocks only exist inside the library's own unit tests
// (`cfg(test)`). An integration test is a separate crate, so it declares
// its own mock of the public trait with `mock!`.
mockall::mock! {
    pub Gateway {}
    impl PaymentGateway for Gateway {
        fn charge(&self, card: &CardToken, amount: Money, idempotency_key: &str)
            -> Result<PaymentId, PaymentError>;
    }
}
