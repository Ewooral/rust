//! The payment provider, behind a trait so tests never charge a real card.

use std::cell::{Cell, RefCell};
use std::fmt;

use crate::money::Money;

/// A token standing for a card (we never see the card number itself).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardToken(pub String);

impl CardToken {
    /// The last four characters, for the receipt.
    pub fn last_four(&self) -> &str {
        let start = self.0.char_indices().rev().nth(3).map_or(0, |(i, _)| i);
        &self.0[start..]
    }
}

/// The provider's reference for a successful charge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentId(pub String);

/// Why a charge failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaymentError {
    /// The bank said no, with its reason.
    Declined(String),
    /// The provider could not be reached.
    Unavailable,
}

impl fmt::Display for PaymentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaymentError::Declined(reason) => write!(f, "card declined: {reason}"),
            PaymentError::Unavailable => write!(f, "payment provider unavailable"),
        }
    }
}

/// Takes money from a card.
///
/// In unit tests (`cfg(test)`) mockall also generates `MockPaymentGateway`
/// from this trait. Integration tests can't see `cfg(test)` items, so
/// `tests/checkout/` declares its own mock with `mockall::mock!`.
#[cfg_attr(test, mockall::automock)]
pub trait PaymentGateway {
    /// Charges `amount` to `card`. The provider uses `idempotency_key` to
    /// ignore a repeated request, so a retry can never charge twice.
    fn charge(
        &self,
        card: &CardToken,
        amount: Money,
        idempotency_key: &str,
    ) -> Result<PaymentId, PaymentError>;
}

/// One charge the fake gateway accepted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Charge {
    /// The card charged.
    pub card: CardToken,
    /// How much.
    pub amount: Money,
    /// The key the caller sent.
    pub idempotency_key: String,
}

/// A **fake** gateway: it really "works" (approves charges, hands out ids,
/// remembers what it did), but entirely in memory. It's also a **spy**:
/// tests can ask it afterwards what happened.
#[derive(Debug, Default)]
pub struct FakeGateway {
    charges: RefCell<Vec<Charge>>,
    declined_cards: Vec<String>,
    next_id: Cell<u32>,
}

impl FakeGateway {
    /// A gateway that approves every card.
    pub fn new() -> FakeGateway {
        FakeGateway::default()
    }

    /// The same, but it declines `card`.
    pub fn declining(mut self, card: &str) -> FakeGateway {
        self.declined_cards.push(card.to_string());
        self
    }

    /// Every accepted charge, oldest first.
    pub fn charges(&self) -> Vec<Charge> {
        self.charges.borrow().clone()
    }
}

impl PaymentGateway for FakeGateway {
    fn charge(
        &self,
        card: &CardToken,
        amount: Money,
        idempotency_key: &str,
    ) -> Result<PaymentId, PaymentError> {
        if self.declined_cards.contains(&card.0) {
            return Err(PaymentError::Declined("insufficient funds".to_string()));
        }
        self.charges.borrow_mut().push(Charge {
            card: card.clone(),
            amount,
            idempotency_key: idempotency_key.to_string(),
        });
        self.next_id.set(self.next_id.get() + 1);
        Ok(PaymentId(format!("pay_{:04}", self.next_id.get())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_four_handles_short_tokens() {
        assert_eq!(CardToken("tok_visa_4242".into()).last_four(), "4242");
        assert_eq!(CardToken("42".into()).last_four(), "42");
    }

    #[test]
    fn fake_gateway_records_and_declines() {
        let gateway = FakeGateway::new().declining("tok_broke");
        let ok = gateway.charge(&CardToken("tok_a".into()), Money::from_pence(5), "k1");
        assert_eq!(ok, Ok(PaymentId("pay_0001".into())));
        let declined = gateway.charge(&CardToken("tok_broke".into()), Money::from_pence(5), "k2");
        assert!(matches!(declined, Err(PaymentError::Declined(_))));
        assert_eq!(
            gateway.charges().len(),
            1,
            "a declined charge is not recorded"
        );
    }
}
