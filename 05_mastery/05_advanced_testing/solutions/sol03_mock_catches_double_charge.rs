//! Solution to exercise 3: make the tests pass. The mock found a bug: fix the CODE.
//!
//! `pay` charges an order and, if the payment provider fails, tries ONE more
//! time. The tests use a mockall mock that knows how many calls to expect.
//! Run them and read mockall's messages: one test says the provider was
//! called more often than expected, another says less often.
//!
//! The bug is one word in `pay`. Charging a customer twice is the kind of
//! mistake that a test with a hand-written stub (which just returns `Ok`)
//! would never notice: a mock checks the CALLS, not only the result.
//! Don't change the tests.
//!
//! Test: cargo test -p advanced_testing --features solutions --bin sol03_mock_catches_double_charge

#[cfg_attr(test, mockall::automock)]
trait Gateway {
    /// Charges `pence`; returns a payment id.
    fn charge(&self, pence: u64, order_id: &str) -> Result<String, String>;
}

const MAX_ATTEMPTS: u32 = 2;

/// Charges the order, trying at most `MAX_ATTEMPTS` times in total.
fn pay(gateway: &impl Gateway, pence: u64, order_id: &str) -> Result<String, String> {
    let mut result = gateway.charge(pence, order_id);
    let mut attempts = 1;
    while attempts < MAX_ATTEMPTS {
        // The bug was `is_err()` here: it stopped after a FAILURE and charged
        // again after a SUCCESS. Stop as soon as a charge goes through.
        if result.is_ok() {
            break; // done: no need to try again
        }
        result = gateway.charge(pence, order_id);
        attempts += 1;
    }
    result
}

struct AlwaysOk;
impl Gateway for AlwaysOk {
    fn charge(&self, pence: u64, order_id: &str) -> Result<String, String> {
        println!("  charged {pence}p for {order_id}");
        Ok(format!("pay_{order_id}"))
    }
}

fn main() {
    println!("{:?}", pay(&AlwaysOk, 915, "ORD-1"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::Sequence;
    use mockall::predicate::eq;

    #[test]
    fn a_successful_payment_charges_exactly_once() {
        let mut gateway = MockGateway::new();
        gateway
            .expect_charge()
            .with(eq(915), eq("ORD-1"))
            .times(1)
            .returning(|_, _| Ok("pay_1".to_string()));

        assert_eq!(pay(&gateway, 915, "ORD-1"), Ok("pay_1".to_string()));
    }

    #[test]
    fn a_failure_is_retried_once() {
        let mut seq = Sequence::new();
        let mut gateway = MockGateway::new();
        gateway
            .expect_charge()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_, _| Err("timeout".to_string()));
        gateway
            .expect_charge()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_, _| Ok("pay_2".to_string()));

        assert_eq!(pay(&gateway, 500, "ORD-2"), Ok("pay_2".to_string()));
    }

    #[test]
    fn gives_up_after_two_failures() {
        let mut gateway = MockGateway::new();
        gateway
            .expect_charge()
            .times(2)
            .returning(|_, _| Err("timeout".to_string()));

        assert_eq!(pay(&gateway, 500, "ORD-3"), Err("timeout".to_string()));
    }
}
