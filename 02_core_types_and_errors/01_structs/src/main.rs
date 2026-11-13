//! Mini project for Book chapter 5: a tiny bank.
//!
//! One struct, an associated function to create it, and methods that read
//! (`&self`), change (`&mut self`) and consume (`self`) it. Amounts are whole
//! pennies in a `u64`, so there is no floating-point rounding.
//!
//! Run: cargo run -p structs    Test: cargo test -p structs

#[derive(Debug, Clone, PartialEq)]
struct Account {
    owner: String,
    balance: u64,
    transactions: u32,
}

impl Account {
    /// A new, empty account.
    fn open(owner: &str) -> Self {
        Self {
            owner: owner.to_string(),
            balance: 0,
            transactions: 0,
        }
    }

    fn owner(&self) -> &str {
        &self.owner
    }

    fn balance(&self) -> u64 {
        self.balance
    }

    fn deposit(&mut self, amount: u64) {
        self.balance += amount;
        self.transactions += 1;
    }

    /// Returns false, and changes nothing, if there is not enough money.
    fn withdraw(&mut self, amount: u64) -> bool {
        if amount > self.balance {
            return false;
        }
        self.balance -= amount;
        self.transactions += 1;
        true
    }

    /// Two different accounts may be borrowed mutably at the same time.
    /// Passing the SAME account twice is E0499, so you cannot pay yourself.
    fn transfer_to(&mut self, other: &mut Account, amount: u64) -> bool {
        if self.withdraw(amount) {
            other.deposit(amount);
            true
        } else {
            false
        }
    }

    /// Closing consumes the account: afterwards it cannot be used at all.
    /// Returns the money left in it.
    fn close(self) -> u64 {
        self.balance
    }
}

/// Formats pennies as pounds: 1234 -> "£12.34".
fn money(pennies: u64) -> String {
    format!("£{}.{:02}", pennies / 100, pennies % 100)
}

fn main() {
    let mut ama = Account::open("Ama");
    let mut kofi = Account::open("Kofi");

    ama.deposit(5_000);
    println!("{} has {}", ama.owner(), money(ama.balance()));

    if ama.transfer_to(&mut kofi, 1_250) {
        println!("sent {} to {}", money(1_250), kofi.owner());
    }
    if !kofi.withdraw(10_000) {
        println!("{} cannot withdraw {}", kofi.owner(), money(10_000));
    }
    // ama.transfer_to(&mut ama, 1); // E0499: cannot borrow `ama` as mutable more than once

    println!("{ama:#?}");
    let left = kofi.close();
    println!("Kofi closed his account and took {}", money(left));
    // kofi.balance(); // E0382: kofi was moved into `close`
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_accounts_are_empty() {
        let a = Account::open("A");
        assert_eq!(a.owner(), "A");
        assert_eq!(a.balance(), 0);
    }

    #[test]
    fn deposit_and_withdraw() {
        let mut a = Account::open("A");
        a.deposit(100);
        assert!(a.withdraw(40));
        assert_eq!(a.balance(), 60);
        assert_eq!(a.transactions, 2);
    }

    #[test]
    fn cannot_overdraw() {
        let mut a = Account::open("A");
        a.deposit(10);
        let before = a.clone();
        assert!(!a.withdraw(11));
        assert_eq!(a, before, "a failed withdrawal must change nothing");
    }

    #[test]
    fn transfer_moves_money_between_accounts() {
        let mut a = Account::open("A");
        let mut b = Account::open("B");
        a.deposit(500);
        assert!(a.transfer_to(&mut b, 200));
        assert_eq!((a.balance(), b.balance()), (300, 200));
        assert!(!a.transfer_to(&mut b, 301));
        assert_eq!((a.balance(), b.balance()), (300, 200));
    }

    #[test]
    fn close_returns_what_is_left() {
        let mut a = Account::open("A");
        a.deposit(99);
        assert_eq!(a.close(), 99);
    }

    #[test]
    fn money_formats_pennies() {
        assert_eq!(money(1234), "£12.34");
        assert_eq!(money(5), "£0.05");
    }
}
