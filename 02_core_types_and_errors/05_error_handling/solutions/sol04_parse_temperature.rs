//! Solution to exercise 4: return a custom error enum.
//!
//! `parse_celsius` turns text like " 21.5 " into a temperature:
//!   - empty or only spaces           -> Err(TempError::Empty)
//!   - not a number                   -> Err(TempError::NotANumber(the trimmed text))
//!   - below absolute zero (-273.15)  -> Err(TempError::BelowAbsoluteZero(value))
//!
//! Then write the Display impl so each error reads well for a person.
//!
//! Test: cargo test -p error_handling --features solutions --bin sol04_parse_temperature

use std::fmt;

#[derive(Debug, PartialEq)]
enum TempError {
    Empty,
    NotANumber(String),
    BelowAbsoluteZero(f64),
}

impl fmt::Display for TempError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TempError::Empty => write!(f, "no temperature given"),
            TempError::NotANumber(text) => write!(f, "{text:?} is not a number"),
            TempError::BelowAbsoluteZero(t) => write!(f, "{t} °C is below absolute zero"),
        }
    }
}

impl std::error::Error for TempError {}

fn parse_celsius(input: &str) -> Result<f64, TempError> {
    let text = input.trim();
    if text.is_empty() {
        return Err(TempError::Empty);
    }
    let value: f64 = text
        .parse()
        .map_err(|_| TempError::NotANumber(text.to_string()))?;
    if value < -273.15 {
        return Err(TempError::BelowAbsoluteZero(value));
    }
    Ok(value)
}

fn main() {
    for input in ["21.5", "", "warm", "-300"] {
        match parse_celsius(input) {
            Ok(t) => println!("{input:?} -> {t} °C"),
            Err(e) => println!("{input:?} -> {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_good_input() {
        assert_eq!(parse_celsius(" 21.5 "), Ok(21.5));
        assert_eq!(parse_celsius("-273.15"), Ok(-273.15));
    }

    #[test]
    fn reports_each_kind_of_error() {
        assert_eq!(parse_celsius("   "), Err(TempError::Empty));
        assert_eq!(
            parse_celsius(" warm "),
            Err(TempError::NotANumber("warm".into()))
        );
        assert_eq!(
            parse_celsius("-300"),
            Err(TempError::BelowAbsoluteZero(-300.0))
        );
    }

    #[test]
    fn messages_are_readable() {
        assert_eq!(TempError::Empty.to_string(), "no temperature given");
        assert_eq!(
            TempError::NotANumber("warm".into()).to_string(),
            "\"warm\" is not a number"
        );
        assert_eq!(
            TempError::BelowAbsoluteZero(-300.0).to_string(),
            "-300 °C is below absolute zero"
        );
    }
}
