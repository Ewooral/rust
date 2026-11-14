//! Money as whole pence: pure logic, tested without a database.
//!
//! Floating-point numbers can't hold most decimal fractions exactly
//! (`0.1 + 0.2 != 0.3`), so prices live in the database as an `INTEGER` number
//! of pence and only become "£12.99" when they're shown to a person.

use std::fmt;
use std::str::FromStr;

/// An amount of money in pence (1/100 of a pound).
///
/// `#[sqlx(transparent)]` makes it travel to and from the database exactly
/// like the `i64` inside it, so it can be bound (`.bind(price)`) and decoded
/// (a `Pence` field in a `FromRow` struct) with no extra code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, sqlx::Type)]
#[sqlx(transparent)]
pub struct Pence(pub i64);

impl Pence {
    /// Multiplies a unit price by a quantity, or `None` on overflow.
    pub fn times(self, quantity: i64) -> Option<Pence> {
        self.0.checked_mul(quantity).map(Pence)
    }
}

impl fmt::Display for Pence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.0 < 0 { "-" } else { "" };
        let abs = self.0.unsigned_abs();
        write!(f, "{sign}£{}.{:02}", abs / 100, abs % 100)
    }
}

/// Why a price string was rejected.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MoneyError {
    /// Nothing to parse.
    #[error("the price is empty")]
    Empty,
    /// Not of the form `12`, `12.5` or `12.99`.
    #[error("{0:?} is not a price like 12.99")]
    Malformed(String),
    /// Too big to store.
    #[error("{0:?} is too large")]
    TooLarge(String),
}

impl FromStr for Pence {
    type Err = MoneyError;

    /// Parses `"12.99"`, `"£12.99"`, `"12.5"` (= 1250p) or `"12"` (= 1200p).
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let trimmed = text.trim();
        let trimmed = trimmed.strip_prefix('£').unwrap_or(trimmed);
        if trimmed.is_empty() {
            return Err(MoneyError::Empty);
        }
        let malformed = || MoneyError::Malformed(text.to_string());
        let (pounds, pence) = match trimmed.split_once('.') {
            Some((pounds, pence)) => (pounds, pence),
            None => (trimmed, ""),
        };
        let all_digits = |s: &str| s.bytes().all(|b| b.is_ascii_digit());
        if pounds.is_empty() || !all_digits(pounds) || pence.len() > 2 || !all_digits(pence) {
            return Err(malformed());
        }
        let pounds: i64 = pounds
            .parse()
            .map_err(|_| MoneyError::TooLarge(text.to_string()))?;
        let pence: i64 = match pence.len() {
            0 => 0,
            1 => pence.parse::<i64>().map_err(|_| malformed())? * 10,
            _ => pence.parse().map_err(|_| malformed())?,
        };
        pounds
            .checked_mul(100)
            .and_then(|p| p.checked_add(pence))
            .map(Pence)
            .ok_or_else(|| MoneyError::TooLarge(text.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_pounds_and_pence() {
        assert_eq!(Pence(1299).to_string(), "£12.99");
        assert_eq!(Pence(5).to_string(), "£0.05");
        assert_eq!(Pence(0).to_string(), "£0.00");
        assert_eq!(Pence(-250).to_string(), "-£2.50");
    }

    #[test]
    fn parses_prices() {
        assert_eq!("12.99".parse(), Ok(Pence(1299)));
        assert_eq!("£8.5".parse(), Ok(Pence(850)));
        assert_eq!(" 7 ".parse(), Ok(Pence(700)));
        assert_eq!("0.05".parse(), Ok(Pence(5)));
    }

    #[test]
    fn rejects_bad_prices() {
        assert_eq!("".parse::<Pence>(), Err(MoneyError::Empty));
        assert!(matches!(
            "12.999".parse::<Pence>(),
            Err(MoneyError::Malformed(_))
        ));
        assert!(matches!(
            "-3".parse::<Pence>(),
            Err(MoneyError::Malformed(_))
        ));
        assert!(matches!(
            "1e3".parse::<Pence>(),
            Err(MoneyError::Malformed(_))
        ));
        assert!(matches!(
            ".50".parse::<Pence>(),
            Err(MoneyError::Malformed(_))
        ));
        assert!(matches!(
            "99999999999999999999".parse::<Pence>(),
            Err(MoneyError::TooLarge(_))
        ));
    }

    #[test]
    fn multiplies_without_overflowing() {
        assert_eq!(Pence(1299).times(3), Some(Pence(3897)));
        assert_eq!(Pence(i64::MAX).times(2), None);
    }
}
