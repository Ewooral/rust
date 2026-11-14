//! ISBN-13 validation: pure logic, tested without a database.
//!
//! Validate input in Rust *before* it reaches SQL. The database's `CHECK`
//! constraints are the last line of defence, not the first.

use std::fmt;
use std::str::FromStr;

/// A valid 13-digit ISBN, stored without hyphens: `9781718503106`.
///
/// The only way to build one from text is [`Isbn::parse`] (or `str::parse`),
/// so every `Isbn` value in the program has already been checked.
/// `#[sqlx(transparent)]` binds and decodes it as the `TEXT` inside.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, sqlx::Type)]
#[sqlx(transparent)]
pub struct Isbn(String);

/// Why an ISBN was rejected.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IsbnError {
    /// Wrong number of digits (after removing hyphens and spaces).
    #[error("an ISBN-13 has 13 digits, found {0}")]
    WrongLength(usize),
    /// A character that is not a digit, hyphen or space.
    #[error("{0:?} is not allowed in an ISBN")]
    BadCharacter(char),
    /// The last digit doesn't match the other twelve.
    #[error("the check digit should be {expected}, not {found}")]
    BadCheckDigit {
        /// The check digit the first twelve digits call for.
        expected: u32,
        /// The check digit that was given.
        found: u32,
    },
}

impl Isbn {
    /// Checks an ISBN-13 such as `"978-1-7185-0310-6"`.
    ///
    /// The check digit makes the weighted sum of all 13 digits (weights
    /// 1, 3, 1, 3, …) a multiple of 10, so a single mistyped digit is caught.
    pub fn parse(text: &str) -> Result<Isbn, IsbnError> {
        let mut digits = Vec::with_capacity(13);
        for c in text.chars() {
            match c {
                '0'..='9' => digits.push(c.to_digit(10).unwrap_or_default()),
                '-' | ' ' => {}
                other => return Err(IsbnError::BadCharacter(other)),
            }
        }
        if digits.len() != 13 {
            return Err(IsbnError::WrongLength(digits.len()));
        }
        let sum: u32 = digits[..12]
            .iter()
            .enumerate()
            .map(|(i, d)| if i % 2 == 0 { *d } else { d * 3 })
            .sum();
        let expected = (10 - sum % 10) % 10;
        let found = digits[12];
        if expected != found {
            return Err(IsbnError::BadCheckDigit { expected, found });
        }
        Ok(Isbn(digits.iter().map(|d| d.to_string()).collect()))
    }

    /// The 13 digits.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for Isbn {
    type Err = IsbnError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Isbn::parse(text)
    }
}

impl fmt::Display for Isbn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_isbns_with_or_without_hyphens() {
        let isbn = Isbn::parse("978-1-7185-0310-6").unwrap();
        assert_eq!(isbn.as_str(), "9781718503106");
        assert_eq!(
            "9780000000019".parse::<Isbn>().unwrap().to_string(),
            "9780000000019"
        );
    }

    #[test]
    fn rejects_wrong_length_and_bad_characters() {
        assert_eq!(Isbn::parse("978-1-7185"), Err(IsbnError::WrongLength(8)));
        assert_eq!(
            Isbn::parse("97817185031X6"),
            Err(IsbnError::BadCharacter('X'))
        );
        assert_eq!(
            Isbn::parse("978' OR '1'='1"),
            Err(IsbnError::BadCharacter('\''))
        );
    }

    #[test]
    fn catches_a_mistyped_digit() {
        assert_eq!(
            Isbn::parse("9781718503105"),
            Err(IsbnError::BadCheckDigit {
                expected: 6,
                found: 5
            })
        );
    }
}
