//! The checkout service: the mini project's code under test.
//!
//! Its three dependencies come in from outside (dependency injection):
//! a [`Clock`] for "now", a [`PaymentGateway`] to take the money, and
//! [`OrderIds`] for random order numbers. That's the whole trick that makes
//! it testable: tests choose the time, fake the bank, and seed the dice.
//!
//! Business rules:
//! - a cart must have at least one line, and no line may have quantity 0;
//! - happy hour (Monday to Friday, 15:00 to 16:59 UTC) takes 10% off;
//! - a coupon takes its percentage off until it expires (`expires_at` is
//!   the first second it no longer works);
//! - discounts don't stack: the bigger one wins (a coupon on a tie);
//! - the card is charged once, with the order id as the idempotency key.

use std::fmt;
use std::fmt::Write as _;

use crate::clock::{Clock, Timestamp, Weekday};
use crate::ids::OrderIds;
use crate::money::Money;
use crate::payment::{CardToken, PaymentError, PaymentGateway, PaymentId};

/// One line of a cart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineItem {
    /// What it is.
    pub name: String,
    /// The price of one.
    pub unit_price: Money,
    /// How many.
    pub quantity: u32,
}

/// What the customer wants to buy.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Cart {
    /// The lines, in the order they were added.
    pub lines: Vec<LineItem>,
}

impl Cart {
    /// An empty cart.
    pub fn new() -> Cart {
        Cart::default()
    }

    /// The cart with one more line (a small builder, handy in tests).
    pub fn with(mut self, name: &str, unit_price: Money, quantity: u32) -> Cart {
        self.lines.push(LineItem {
            name: name.to_string(),
            unit_price,
            quantity,
        });
        self
    }
}

/// A discount code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Coupon {
    /// What the customer types, e.g. `WELCOME10`.
    pub code: String,
    /// Percentage off, 1 to 100.
    pub percent_off: u8,
    /// The first moment the coupon no longer works.
    pub expires_at: Timestamp,
}

/// Why an order was not placed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckoutError {
    /// Nothing to buy.
    EmptyCart,
    /// A line with quantity 0 (its name).
    ZeroQuantity(String),
    /// The total doesn't fit in a `Money`.
    TooLarge,
    /// No coupon with this code.
    UnknownCoupon(String),
    /// The coupon exists but has expired.
    CouponExpired(String),
    /// The payment failed; nothing was charged.
    Payment(PaymentError),
}

impl fmt::Display for CheckoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckoutError::EmptyCart => write!(f, "the cart is empty"),
            CheckoutError::ZeroQuantity(name) => write!(f, "{name}: quantity must be at least 1"),
            CheckoutError::TooLarge => write!(f, "the order total is too large"),
            CheckoutError::UnknownCoupon(code) => write!(f, "no such coupon: {code}"),
            CheckoutError::CouponExpired(code) => write!(f, "coupon {code} has expired"),
            CheckoutError::Payment(e) => write!(f, "payment failed: {e}"),
        }
    }
}

impl std::error::Error for CheckoutError {}

/// A placed order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt {
    /// The order number.
    pub order_id: String,
    /// When it was placed.
    pub placed_at: Timestamp,
    /// What was bought.
    pub lines: Vec<LineItem>,
    /// Before any discount.
    pub subtotal: Money,
    /// The discount applied, if any: a label and the amount.
    pub discount: Option<(String, Money)>,
    /// What was charged.
    pub total: Money,
    /// The card's last four characters.
    pub card_last_four: String,
    /// The payment provider's reference.
    pub payment_id: PaymentId,
}

impl Receipt {
    /// The receipt as plain text, 40 columns wide.
    pub fn render(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "Receipt {}", self.order_id);
        let _ = writeln!(out, "{}", self.placed_at);
        out.push('\n');
        for line in &self.lines {
            let cost = line
                .unit_price
                .checked_mul(line.quantity)
                .unwrap_or(Money::ZERO);
            let item = format!("{} x {}", line.quantity, line.name);
            let _ = writeln!(
                out,
                "{item:<22}{:>8}{cost:>10}",
                line.unit_price.to_string()
            );
        }
        out.push('\n');
        let _ = writeln!(out, "{:<30}{:>10}", "Subtotal", self.subtotal.to_string());
        if let Some((label, amount)) = &self.discount {
            let _ = writeln!(out, "{label:<30}{:>10}", format!("-{amount}"));
        }
        let _ = writeln!(out, "{:<30}{:>10}", "Total", self.total.to_string());
        let _ = writeln!(
            out,
            "Paid by card ending {} ({})",
            self.card_last_four, self.payment_id.0
        );
        out
    }
}

/// Monday to Friday, 15:00 to 16:59 UTC.
pub fn is_happy_hour(at: Timestamp) -> bool {
    let weekday = !matches!(at.weekday(), Weekday::Saturday | Weekday::Sunday);
    weekday && (15..17).contains(&at.hour())
}

/// The happy-hour discount, in percent.
pub const HAPPY_HOUR_PERCENT: u8 = 10;

/// The checkout service, generic over its clock and payment gateway.
pub struct Checkout<C, P> {
    clock: C,
    payments: P,
    ids: OrderIds,
    coupons: Vec<Coupon>,
}

impl<C: Clock, P: PaymentGateway> Checkout<C, P> {
    /// A checkout with no coupons.
    pub fn new(clock: C, payments: P, ids: OrderIds) -> Self {
        Checkout {
            clock,
            payments,
            ids,
            coupons: Vec::new(),
        }
    }

    /// Adds a coupon customers can use.
    pub fn with_coupon(mut self, coupon: Coupon) -> Self {
        self.coupons.push(coupon);
        self
    }

    /// The payment gateway, so a test can inspect a fake afterwards.
    pub fn payments(&self) -> &P {
        &self.payments
    }

    /// Validates the cart, works out the discount, charges the card once,
    /// and returns the receipt.
    ///
    /// # Errors
    ///
    /// Every [`CheckoutError`] except `Payment` is returned **before** the
    /// card is touched.
    pub fn place_order(
        &mut self,
        cart: &Cart,
        card: &CardToken,
        coupon_code: Option<&str>,
    ) -> Result<Receipt, CheckoutError> {
        if cart.lines.is_empty() {
            return Err(CheckoutError::EmptyCart);
        }
        let mut subtotal = Money::ZERO;
        for line in &cart.lines {
            if line.quantity == 0 {
                return Err(CheckoutError::ZeroQuantity(line.name.clone()));
            }
            subtotal = line
                .unit_price
                .checked_mul(line.quantity)
                .and_then(|cost| subtotal.checked_add(cost))
                .ok_or(CheckoutError::TooLarge)?;
        }

        let now = self.clock.now();
        let discount = self.best_discount(subtotal, now, coupon_code)?;
        let total = match &discount {
            Some((_, amount)) => subtotal.saturating_sub(*amount),
            None => subtotal,
        };

        let order_id = self.ids.next_id();
        let payment_id = self
            .payments
            .charge(card, total, &order_id)
            .map_err(CheckoutError::Payment)?;

        Ok(Receipt {
            order_id,
            placed_at: now,
            lines: cart.lines.clone(),
            subtotal,
            discount,
            total,
            card_last_four: card.last_four().to_string(),
            payment_id,
        })
    }

    fn best_discount(
        &self,
        subtotal: Money,
        now: Timestamp,
        coupon_code: Option<&str>,
    ) -> Result<Option<(String, Money)>, CheckoutError> {
        let coupon = match coupon_code {
            None => None,
            Some(code) => {
                let coupon = self
                    .coupons
                    .iter()
                    .find(|c| c.code.eq_ignore_ascii_case(code))
                    .ok_or_else(|| CheckoutError::UnknownCoupon(code.to_string()))?;
                if now >= coupon.expires_at {
                    return Err(CheckoutError::CouponExpired(coupon.code.clone()));
                }
                Some((
                    format!("Coupon {} ({}%)", coupon.code, coupon.percent_off),
                    subtotal.percent(coupon.percent_off),
                ))
            }
        };
        let happy_hour = is_happy_hour(now).then(|| {
            (
                format!("Happy hour ({HAPPY_HOUR_PERCENT}%)"),
                subtotal.percent(HAPPY_HOUR_PERCENT),
            )
        });
        Ok(match (coupon, happy_hour) {
            (Some(c), Some(h)) => Some(if h.1 > c.1 { h } else { c }),
            (c, h) => c.or(h),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::FixedClock;
    use crate::payment::MockPaymentGateway;
    use mockall::predicate::{always, eq};

    // Test data with names that say what matters about them.
    fn monday_morning() -> Timestamp {
        Timestamp::from_utc(2026, 11, 16, 9, 0)
    }

    fn monday_happy_hour() -> Timestamp {
        Timestamp::from_utc(2026, 11, 16, 15, 30)
    }

    fn card() -> CardToken {
        CardToken("tok_visa_4242".to_string())
    }

    fn coffee_and_cake() -> Cart {
        Cart::new()
            .with("Flat white", Money::from_pence(320), 2)
            .with("Banana bread", Money::from_pence(275), 1)
    }

    fn checkout_at(
        now: Timestamp,
        gateway: MockPaymentGateway,
    ) -> Checkout<FixedClock, MockPaymentGateway> {
        Checkout::new(FixedClock::at(now), gateway, OrderIds::seeded(1))
    }

    // Arrange, act, assert: one behaviour per test, named after it.
    #[test]
    fn charges_the_subtotal_once_outside_happy_hour() {
        // Arrange: the mock EXPECTS exactly one charge of £9.15.
        let mut gateway = MockPaymentGateway::new();
        gateway
            .expect_charge()
            .with(eq(card()), eq(Money::from_pence(915)), always())
            .times(1)
            .returning(|_, _, _| Ok(PaymentId("pay_1".into())));
        let mut checkout = checkout_at(monday_morning(), gateway);

        // Act
        let receipt = checkout
            .place_order(&coffee_and_cake(), &card(), None)
            .unwrap();

        // Assert (and when `checkout` is dropped, mockall checks times(1)).
        assert_eq!(receipt.total, Money::from_pence(915));
        assert_eq!(receipt.discount, None);
    }

    #[test]
    fn happy_hour_takes_ten_percent_off() {
        let mut gateway = MockPaymentGateway::new();
        gateway
            .expect_charge()
            .with(always(), eq(Money::from_pence(824)), always())
            .times(1)
            .returning(|_, _, _| Ok(PaymentId("pay_1".into())));
        let mut checkout = checkout_at(monday_happy_hour(), gateway);

        let receipt = checkout
            .place_order(&coffee_and_cake(), &card(), None)
            .unwrap();

        assert_eq!(
            receipt.discount,
            Some(("Happy hour (10%)".to_string(), Money::from_pence(91)))
        );
    }

    #[test]
    fn happy_hour_ends_at_five_and_skips_weekends() {
        assert!(is_happy_hour(Timestamp::from_utc(2026, 11, 20, 16, 59))); // Friday
        assert!(!is_happy_hour(Timestamp::from_utc(2026, 11, 20, 17, 0)));
        assert!(!is_happy_hour(Timestamp::from_utc(2026, 11, 21, 15, 30))); // Saturday
    }

    #[test]
    fn invalid_carts_never_reach_the_payment_gateway() {
        let mut gateway = MockPaymentGateway::new();
        gateway.expect_charge().never();
        let mut checkout = checkout_at(monday_morning(), gateway);

        let empty = checkout.place_order(&Cart::new(), &card(), None);
        let zero = checkout.place_order(
            &Cart::new().with("Tea", Money::from_pence(200), 0),
            &card(),
            None,
        );
        let unknown = checkout.place_order(&coffee_and_cake(), &card(), Some("NOPE"));

        assert_eq!(empty, Err(CheckoutError::EmptyCart));
        assert_eq!(zero, Err(CheckoutError::ZeroQuantity("Tea".into())));
        assert_eq!(unknown, Err(CheckoutError::UnknownCoupon("NOPE".into())));
    }

    #[test]
    fn a_declined_card_is_reported_and_no_receipt_is_made() {
        let mut gateway = MockPaymentGateway::new();
        gateway
            .expect_charge()
            .times(1)
            .returning(|_, _, _| Err(PaymentError::Declined("insufficient funds".into())));
        let mut checkout = checkout_at(monday_morning(), gateway);

        let result = checkout.place_order(&coffee_and_cake(), &card(), None);

        assert_eq!(
            result,
            Err(CheckoutError::Payment(PaymentError::Declined(
                "insufficient funds".into()
            )))
        );
    }

    #[test]
    fn the_order_id_is_the_idempotency_key() {
        let mut gateway = MockPaymentGateway::new();
        gateway
            .expect_charge()
            .withf(|_, _, key| key.starts_with("ORD-") && key.len() == 10)
            .times(1)
            .returning(|_, _, key| Ok(PaymentId(format!("pay_for_{key}"))));
        let mut checkout = checkout_at(monday_morning(), gateway);

        let receipt = checkout
            .place_order(&coffee_and_cake(), &card(), None)
            .unwrap();

        assert_eq!(
            receipt.payment_id.0,
            format!("pay_for_{}", receipt.order_id)
        );
    }

    #[test]
    fn a_coupon_works_until_the_second_it_expires() {
        let expires = Timestamp::from_utc(2026, 11, 30, 0, 0);
        let clock = FixedClock::at(Timestamp(expires.0 - 1)); // one second before
        let mut gateway = MockPaymentGateway::new();
        gateway
            .expect_charge()
            .times(1)
            .returning(|_, _, _| Ok(PaymentId("pay_1".into())));
        let coupon = Coupon {
            code: "WELCOME20".into(),
            percent_off: 20,
            expires_at: expires,
        };
        let mut checkout = Checkout::new(&clock, gateway, OrderIds::seeded(1)).with_coupon(coupon);

        let before = checkout.place_order(&coffee_and_cake(), &card(), Some("welcome20"));
        clock.advance(1); // exactly at expiry
        let at_expiry = checkout.place_order(&coffee_and_cake(), &card(), Some("WELCOME20"));

        assert_eq!(before.unwrap().total, Money::from_pence(732));
        assert_eq!(
            at_expiry,
            Err(CheckoutError::CouponExpired("WELCOME20".into()))
        );
    }

    #[test]
    fn the_bigger_discount_wins() {
        let mut gateway = MockPaymentGateway::new();
        gateway
            .expect_charge()
            .times(2)
            .returning(|_, _, _| Ok(PaymentId("pay".into())));
        let far_future = Timestamp::from_utc(2030, 1, 1, 0, 0);
        let mut checkout = checkout_at(monday_happy_hour(), gateway)
            .with_coupon(Coupon {
                code: "SMALL5".into(),
                percent_off: 5,
                expires_at: far_future,
            })
            .with_coupon(Coupon {
                code: "BIG25".into(),
                percent_off: 25,
                expires_at: far_future,
            });

        let small = checkout
            .place_order(&coffee_and_cake(), &card(), Some("SMALL5"))
            .unwrap();
        let big = checkout
            .place_order(&coffee_and_cake(), &card(), Some("BIG25"))
            .unwrap();

        assert_eq!(small.discount.unwrap().0, "Happy hour (10%)");
        assert_eq!(big.discount.unwrap().0, "Coupon BIG25 (25%)");
    }

    #[test]
    fn a_total_too_large_for_money_is_an_error() {
        let mut gateway = MockPaymentGateway::new();
        gateway.expect_charge().never();
        let mut checkout = checkout_at(monday_morning(), gateway);
        let cart = Cart::new().with("Yacht", Money::from_pence(u64::MAX), 2);

        assert_eq!(
            checkout.place_order(&cart, &card(), None),
            Err(CheckoutError::TooLarge)
        );
    }
}
