//! Exercise 4: operator overloading on a newtype.
//!
//! `Money` stores whole pence, so it never has floating-point rounding
//! errors. Fill in each `todo!()` so that:
//!   - `a + b`, `a - b` and `-a` work (Add, Sub, Neg),
//!   - `price * 3` works (Mul<u32>: Rhs is NOT Self here),
//!   - `total += price` works (AddAssign),
//!   - an iterator of Money can `.sum()` (Sum),
//!   - it prints as `£12.05`, or `-£0.50` when negative (Display).
//!
//! The tests are written; make them pass.
//! Hint for Display: `self.pence.abs() / 100` and `% 100`, with `{:02}`.
//!
//! Test: cargo test -p advanced_features --features exercises --bin ex04_money_operators

#![allow(unused_variables)]

use std::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Money {
    pence: i64,
}

impl Money {
    fn new(pounds: i64, pence: i64) -> Money {
        Money {
            pence: pounds * 100 + pence,
        }
    }
}

impl Add for Money {
    type Output = Money;
    fn add(self, other: Money) -> Money {
        todo!()
    }
}

impl Sub for Money {
    type Output = Money;
    fn sub(self, other: Money) -> Money {
        todo!()
    }
}

impl Neg for Money {
    type Output = Money;
    fn neg(self) -> Money {
        todo!()
    }
}

impl Mul<u32> for Money {
    type Output = Money;
    fn mul(self, count: u32) -> Money {
        todo!()
    }
}

impl AddAssign for Money {
    fn add_assign(&mut self, other: Money) {
        todo!()
    }
}

impl Sum for Money {
    fn sum<I: Iterator<Item = Money>>(iter: I) -> Money {
        todo!()
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

fn main() {
    let coffee = Money::new(3, 20);
    let cake = Money::new(2, 85);
    let order = coffee * 2 + cake;
    println!("2 coffees and a cake: {order}");
    println!("change from a tenner: {}", Money::new(10, 0) - order);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_sub_neg_and_mul() {
        let a = Money::new(1, 50);
        let b = Money::new(0, 75);
        assert_eq!(a + b, Money::new(2, 25));
        assert_eq!(a - b, Money::new(0, 75));
        assert_eq!(-b, Money { pence: -75 });
        assert_eq!(b * 4, Money::new(3, 0));
    }

    #[test]
    fn add_assign_and_sum() {
        let mut total = Money::new(0, 0);
        total += Money::new(1, 1);
        total += Money::new(2, 2);
        assert_eq!(total, Money::new(3, 3));
        let basket = [Money::new(1, 0), Money::new(0, 99), Money::new(5, 1)];
        assert_eq!(basket.into_iter().sum::<Money>(), Money::new(7, 0));
    }

    #[test]
    fn displays_pounds_and_pence() {
        assert_eq!(Money::new(12, 5).to_string(), "£12.05");
        assert_eq!(Money::new(0, 0).to_string(), "£0.00");
        assert_eq!(Money { pence: -50 }.to_string(), "-£0.50");
        assert_eq!(Money::new(-3, -10).to_string(), "-£3.10");
    }
}
