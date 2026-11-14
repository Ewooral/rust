//! Solution to exercise 4: write the tests, with mockall mocks.
//!
//! `place_order` is correct. It reserves stock, then charges the card; if
//! the charge fails, it releases the stock again. Write the three tests
//! (replace each `todo!`) so that each one checks what its NAME promises:
//! - set up `MockInventory` and `MockPayments` with `expect_...()`,
//!   `.with(eq(..), eq(..))`, `.times(n)` or `.never()`, and `.returning(..)`;
//! - call `place_order(&inventory, &payments, "TEA", 2, 500)`;
//! - assert on the result.
//!
//! A good mock test fails if the code does a call it shouldn't (`never()`),
//! or forgets one it should (`times(1)`). Check yours: break `place_order`
//! on purpose (delete the `release` line) and make sure a test goes red.
//!
//! Test: cargo test -p advanced_testing --features solutions --bin sol04_write_the_mocks

#[cfg_attr(test, mockall::automock)]
trait Inventory {
    fn reserve(&self, sku: &str, quantity: u32) -> Result<(), String>;
    fn release(&self, sku: &str, quantity: u32);
}

#[cfg_attr(test, mockall::automock)]
trait Payments {
    fn charge(&self, pence: u64) -> Result<String, String>;
}

/// Reserve, then charge; undo the reservation if the charge fails.
fn place_order(
    inventory: &dyn Inventory,
    payments: &dyn Payments,
    sku: &str,
    quantity: u32,
    pence: u64,
) -> Result<String, String> {
    inventory.reserve(sku, quantity)?;
    match payments.charge(pence) {
        Ok(payment_id) => Ok(payment_id),
        Err(e) => {
            inventory.release(sku, quantity);
            Err(e)
        }
    }
}

struct Shop;
impl Inventory for Shop {
    fn reserve(&self, sku: &str, quantity: u32) -> Result<(), String> {
        println!("  reserved {quantity} x {sku}");
        Ok(())
    }
    fn release(&self, sku: &str, quantity: u32) {
        println!("  released {quantity} x {sku}");
    }
}
impl Payments for Shop {
    fn charge(&self, pence: u64) -> Result<String, String> {
        println!("  charged {pence}p");
        Ok("pay_1".to_string())
    }
}

fn main() {
    println!("{:?}", place_order(&Shop, &Shop, "TEA", 2, 500));
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::eq;

    #[test]
    fn reserves_the_stock_then_charges_the_card() {
        let mut inventory = MockInventory::new();
        inventory
            .expect_reserve()
            .with(eq("TEA"), eq(2))
            .times(1)
            .returning(|_, _| Ok(()));
        inventory.expect_release().never();
        let mut payments = MockPayments::new();
        payments
            .expect_charge()
            .with(eq(500))
            .times(1)
            .returning(|_| Ok("pay_1".to_string()));

        let result = place_order(&inventory, &payments, "TEA", 2, 500);

        assert_eq!(result, Ok("pay_1".to_string()));
    }

    #[test]
    fn does_not_charge_when_the_stock_is_not_there() {
        let mut inventory = MockInventory::new();
        inventory
            .expect_reserve()
            .returning(|_, _| Err("out of stock".to_string()));
        inventory.expect_release().never();
        let mut payments = MockPayments::new();
        payments.expect_charge().never();

        let result = place_order(&inventory, &payments, "TEA", 2, 500);

        assert_eq!(result, Err("out of stock".to_string()));
    }

    #[test]
    fn releases_the_stock_when_the_charge_fails() {
        let mut inventory = MockInventory::new();
        inventory.expect_reserve().returning(|_, _| Ok(()));
        inventory
            .expect_release()
            .with(eq("TEA"), eq(2))
            .times(1)
            .return_const(());
        let mut payments = MockPayments::new();
        payments
            .expect_charge()
            .returning(|_| Err("card declined".to_string()));

        let result = place_order(&inventory, &payments, "TEA", 2, 500);

        assert_eq!(result, Err("card declined".to_string()));
    }
}
