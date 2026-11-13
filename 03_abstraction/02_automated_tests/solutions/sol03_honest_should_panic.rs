//! Solution to exercise 3: a test that passes for the WRONG reason.
//!
//! Run the tests: both pass. But look closely at `cannot_overdraw`: it calls
//! withdraw with an amount of 0, so it panics because of the zero, not
//! because of the overdraft it claims to test.
//!   1. Add `expected = "insufficient funds"` to its #[should_panic]. Run
//!      again: now it fails, and the report shows the real panic message.
//!   2. Fix the TEST's input so it really tries to overdraw.
//!
//! Test: cargo test -p automated_tests --features solutions --bin sol03_honest_should_panic

fn withdraw(balance: u32, amount: u32) -> u32 {
    if amount == 0 {
        panic!("amount must be positive");
    }
    if amount > balance {
        panic!("insufficient funds: balance {balance}, asked for {amount}");
    }
    balance - amount
}

fn main() {
    println!("100 - 30 = {}", withdraw(100, 30));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_withdrawal() {
        assert_eq!(withdraw(100, 30), 70);
    }

    #[test]
    #[should_panic(expected = "insufficient funds")]
    fn cannot_overdraw() {
        withdraw(10, 20);
    }

    // The zero case deserves its own honest test.
    #[test]
    #[should_panic(expected = "must be positive")]
    fn zero_is_rejected() {
        withdraw(10, 0);
    }
}
