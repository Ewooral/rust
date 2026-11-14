//! Exercise 1: make it compile (E0061) by injecting a clock.
//!
//! `Coupon::is_valid` asks the operating system what time it is. To test
//! "the coupon stops working at midnight", you'd have to run the test at
//! midnight. The tests below (already written) hand the method a clock
//! instead: `coupon.is_valid(&clock)`. That doesn't compile yet.
//!
//! Change `is_valid` to take `clock: &impl Clock` and use `clock.now()`
//! (import `Clock` from `advanced_testing::clock`), then delete the
//! `SystemTime` code. `main` already passes the real `SystemClock`; the
//! tests pass a `FixedClock` set to exactly the moment they care about.
//!
//! Test: cargo test -p advanced_testing --features exercises --bin ex01_inject_the_clock

use std::time::{SystemTime, UNIX_EPOCH};

use advanced_testing::clock::{SystemClock, Timestamp};

struct Coupon {
    code: String,
    /// The first second the coupon no longer works.
    expires_at: Timestamp,
}

impl Coupon {
    fn is_valid(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before 1970")
            .as_secs();
        now < self.expires_at.0
    }
}

fn welcome() -> Coupon {
    Coupon {
        code: "WELCOME20".to_string(),
        expires_at: Timestamp::from_utc(2026, 12, 1, 0, 0),
    }
}

fn main() {
    let coupon = welcome();
    println!(
        "{} valid right now: {}",
        coupon.code,
        coupon.is_valid(&SystemClock)
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use advanced_testing::clock::FixedClock;

    #[test]
    fn valid_one_second_before_it_expires() {
        let clock = FixedClock::at(Timestamp::from_utc(2026, 11, 30, 23, 59).plus_seconds(59));
        assert!(welcome().is_valid(&clock));
    }

    #[test]
    fn invalid_from_the_second_it_expires() {
        let clock = FixedClock::at(Timestamp::from_utc(2026, 12, 1, 0, 0));
        assert!(!welcome().is_valid(&clock));
    }

    #[test]
    fn invalid_years_later() {
        let clock = FixedClock::at(Timestamp::from_utc(2030, 6, 1, 12, 0));
        assert!(!welcome().is_valid(&clock));
    }
}
