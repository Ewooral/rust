//! Money in whole pence, and a parser that must never panic.
//!
//! Money is never an `f64`: 0.1 + 0.2 is not 0.3 in floating point. Whole
//! pence in a `u64` are exact. The parser is the lesson's favourite target
//! for property tests and fuzzing, because it takes text from outside.

use std::fmt;

/// An amount of money in pence (1/100 of a pound).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Money(u64);

impl Money {
    /// Nothing at all.
    pub const ZERO: Money = Money(0);

    /// An amount in pence: `Money::from_pence(250)` is £2.50.
    pub const fn from_pence(pence: u64) -> Money {
        Money(pence)
    }

    /// The amount in pence.
    pub const fn pence(self) -> u64 {
        self.0
    }

    /// `self + other`, or `None` if it would overflow.
    pub fn checked_add(self, other: Money) -> Option<Money> {
        self.0.checked_add(other.0).map(Money)
    }

    /// `self * quantity`, or `None` if it would overflow.
    pub fn checked_mul(self, quantity: u32) -> Option<Money> {
        self.0.checked_mul(u64::from(quantity)).map(Money)
    }

    /// `self - other`, or zero if `other` is bigger.
    pub fn saturating_sub(self, other: Money) -> Money {
        Money(self.0.saturating_sub(other.0))
    }

    /// `percent`% of this amount, rounded down to whole pence.
    ///
    /// ```
    /// use advanced_testing::money::Money;
    /// assert_eq!(Money::from_pence(915).percent(10), Money::from_pence(91));
    /// ```
    pub fn percent(self, percent: u8) -> Money {
        // u128 so that even u64::MAX * 100 can't overflow.
        Money((u128::from(self.0) * u128::from(percent) / 100) as u64)
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = format!("£{}.{:02}", self.0 / 100, self.0 % 100);
        f.pad(&text) // so width and alignment, like {:>8}, work
    }
}

/// Why some text is not an amount of money.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoneyError {
    /// Nothing to parse.
    Empty,
    /// A character that isn't a digit (or a misplaced `£` or `.`).
    InvalidCharacter(char),
    /// A `.` not followed by exactly two digits.
    BadPence,
    /// More than `u64::MAX` pence.
    TooLarge,
}

impl fmt::Display for MoneyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoneyError::Empty => write!(f, "no amount given"),
            MoneyError::InvalidCharacter(c) => write!(f, "unexpected character {c:?}"),
            MoneyError::BadPence => write!(f, "pence must be exactly two digits"),
            MoneyError::TooLarge => write!(f, "amount is too large"),
        }
    }
}

impl std::error::Error for MoneyError {}

/// Parses `"12"`, `"12.30"`, `"£12.30"` (surrounding spaces allowed).
///
/// It never panics, whatever the input: every arithmetic step is checked.
///
/// ```
/// use advanced_testing::money::{Money, MoneyError, parse_money};
/// assert_eq!(parse_money("£3.20"), Ok(Money::from_pence(320)));
/// assert_eq!(parse_money("3.2"), Err(MoneyError::BadPence));
/// ```
pub fn parse_money(text: &str) -> Result<Money, MoneyError> {
    let text = text.trim();
    let text = text.strip_prefix('£').unwrap_or(text);
    if text.is_empty() {
        return Err(MoneyError::Empty);
    }
    let (pounds, pence) = match text.split_once('.') {
        Some((pounds, pence)) => (pounds, Some(pence)),
        None => (text, None),
    };
    if pounds.is_empty() {
        return Err(MoneyError::Empty);
    }
    let mut total: u64 = 0;
    for c in pounds.chars() {
        let digit = c.to_digit(10).ok_or(MoneyError::InvalidCharacter(c))?;
        total = total
            .checked_mul(10)
            .and_then(|t| t.checked_add(u64::from(digit)))
            .ok_or(MoneyError::TooLarge)?;
    }
    let mut cents: u64 = 0;
    if let Some(pence) = pence {
        if pence.chars().count() != 2 {
            return Err(MoneyError::BadPence);
        }
        for c in pence.chars() {
            let digit = c.to_digit(10).ok_or(MoneyError::InvalidCharacter(c))?;
            cents = cents * 10 + u64::from(digit);
        }
    }
    total
        .checked_mul(100)
        .and_then(|t| t.checked_add(cents))
        .map(Money)
        .ok_or(MoneyError::TooLarge)
}

/// The invariants a fuzzer checks on every input (see `tests/checkout/fuzz.rs`
/// and the README). Panics if one is broken; otherwise does nothing.
///
/// 1. `parse_money` returns, it never panics;
/// 2. whatever parses, prints, and parses back to the same amount.
pub fn check_parse_money(text: &str) {
    if let Ok(money) = parse_money(text) {
        let printed = money.to_string();
        assert_eq!(
            parse_money(&printed),
            Ok(money),
            "{text:?} parsed to {money:?}, printed as {printed:?}, which did not parse back"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pounds_and_pence() {
        assert_eq!(parse_money("12"), Ok(Money::from_pence(1_200)));
        assert_eq!(parse_money(" £0.05 "), Ok(Money::from_pence(5)));
    }

    #[test]
    fn rejects_bad_input_with_a_reason() {
        assert_eq!(parse_money(""), Err(MoneyError::Empty));
        assert_eq!(parse_money("£"), Err(MoneyError::Empty));
        assert_eq!(parse_money(".50"), Err(MoneyError::Empty));
        assert_eq!(parse_money("1.5"), Err(MoneyError::BadPence));
        assert_eq!(parse_money("1.505"), Err(MoneyError::BadPence));
        assert_eq!(parse_money("1,50"), Err(MoneyError::InvalidCharacter(',')));
        assert_eq!(parse_money("-1"), Err(MoneyError::InvalidCharacter('-')));
    }

    #[test]
    fn huge_amounts_are_an_error_not_a_panic() {
        assert_eq!(parse_money("184467440737095516.15"), Ok(Money(u64::MAX)));
        assert_eq!(
            parse_money("184467440737095516.16"),
            Err(MoneyError::TooLarge)
        );
        assert_eq!(
            parse_money("99999999999999999999"),
            Err(MoneyError::TooLarge)
        );
    }

    #[test]
    fn displays_with_two_decimals_and_honours_width() {
        assert_eq!(Money::from_pence(5).to_string(), "£0.05");
        assert_eq!(format!("[{:>7}]", Money::from_pence(1_234)), "[ £12.34]");
    }

    #[test]
    fn percent_rounds_down() {
        assert_eq!(Money::from_pence(999).percent(10), Money::from_pence(99));
        assert_eq!(Money(u64::MAX).percent(100), Money(u64::MAX));
    }

    #[test]
    fn non_ascii_digits_are_rejected() {
        // '٣' is ARABIC-INDIC DIGIT THREE: is_numeric() says yes, but
        // to_digit(10) only accepts 0-9.
        assert_eq!(parse_money("٣"), Err(MoneyError::InvalidCharacter('٣')));
    }
}
