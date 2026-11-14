//! Lesson 05/05: advanced testing.
//!
//! The mini project is a checkout service built to be tested: its clock,
//! payment provider and random order ids are all injected, so tests can
//! control every one of them.
//!
//! - [`checkout`]: the service, its rules, and the receipt.
//! - [`clock`]: the `Clock` trait, `SystemClock` and the `FixedClock` fake.
//! - [`payment`]: the `PaymentGateway` trait and the `FakeGateway` fake.
//! - [`ids`]: order ids from a seedable random generator.
//! - [`money`]: whole-pence money and a parser that never panics.
//!
//! Tests live in three places: unit tests next to the code (`src/`), one
//! integration-test binary in `tests/checkout/`, and doc tests like this:
//!
//! ```
//! use advanced_testing::checkout::{Cart, Checkout};
//! use advanced_testing::clock::{FixedClock, Timestamp};
//! use advanced_testing::ids::OrderIds;
//! use advanced_testing::money::Money;
//! use advanced_testing::payment::{CardToken, FakeGateway};
//!
//! let saturday = Timestamp::from_utc(2026, 11, 21, 12, 0);
//! let mut checkout = Checkout::new(FixedClock::at(saturday), FakeGateway::new(), OrderIds::seeded(1));
//! let cart = Cart::new().with("Tea", Money::from_pence(250), 2);
//! let receipt = checkout.place_order(&cart, &CardToken("tok_1234".into()), None)?;
//! assert_eq!(receipt.total, Money::from_pence(500));
//! # Ok::<(), advanced_testing::checkout::CheckoutError>(())
//! ```

pub mod checkout;
pub mod clock;
pub mod ids;
pub mod money;
pub mod payment;
