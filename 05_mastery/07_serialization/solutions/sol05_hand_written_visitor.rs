//! Solution to exercise 5: write a Visitor (pass 3 tests).
//!
//! A weather station's files store temperatures in two ways:
//!
//!     "outside": 21.5          a number: degrees Celsius
//!     "inside": "70F"          a string: a number and a unit, C or F
//!
//! `Temperature` holds Celsius. Its `Serialize` impl is done (it writes a
//! number). Write its `Deserialize` impl, by hand:
//!
//!   1. `deserialize`: hand the deserializer a `TemperatureVisitor` with
//!      `deserialize_any` (we accept more than one kind of value).
//!   2. `expecting`: finish the sentence "expected ___" with
//!      `a temperature in Celsius, or a string like "21.5C" or "70F"`.
//!   3. `visit_f64`, `visit_i64`, `visit_u64`: a number is already Celsius.
//!      (JSON gives 21.5 as f64 and 21 as u64; TOML gives 21 as i64.)
//!   4. `visit_str`: split off the last character as the unit. "C" keeps
//!      the number, "F" converts: (f - 32) * 5 / 9. Anything else, or a
//!      number that doesn't parse, is
//!      `Err(E::invalid_value(Unexpected::Str(text), &self))`.
//!
//! Test: cargo test -p serialization --features solutions --bin sol05_hand_written_visitor

use std::fmt;

use serde::de::{self, Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Temperature {
    celsius: f64,
}

impl Serialize for Temperature {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.celsius)
    }
}

impl<'de> Deserialize<'de> for Temperature {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Temperature, D::Error> {
        deserializer.deserialize_any(TemperatureVisitor)
    }
}

struct TemperatureVisitor;

impl Visitor<'_> for TemperatureVisitor {
    type Value = Temperature;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a temperature in Celsius, or a string like \"21.5C\" or \"70F\"")
    }

    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Temperature, E> {
        Ok(Temperature { celsius: value })
    }

    // `as f64` is fine for temperatures: no thermometer reads past 2^53.
    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Temperature, E> {
        self.visit_f64(value as f64)
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Temperature, E> {
        self.visit_f64(value as f64)
    }

    fn visit_str<E: de::Error>(self, text: &str) -> Result<Temperature, E> {
        let invalid = || E::invalid_value(Unexpected::Str(text), &self);
        // Slicing at len() - 1 would panic inside a multi-byte character;
        // `char_indices().last()` finds where the last character really starts.
        let (cut, _) = text.char_indices().last().ok_or_else(invalid)?;
        let (number, unit) = text.split_at(cut);
        let number: f64 = number.parse().map_err(|_| invalid())?;
        match unit {
            "C" => Ok(Temperature { celsius: number }),
            "F" => Ok(Temperature {
                celsius: (number - 32.0) * 5.0 / 9.0,
            }),
            _ => Err(invalid()),
        }
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Reading {
    outside: Temperature,
    inside: Temperature,
}

fn main() {
    let reading: Reading = serde_json::from_str(r#"{"outside": 21.5, "inside": "70F"}"#).unwrap();
    println!("{reading:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn celsius(json: &str) -> f64 {
        serde_json::from_str::<Temperature>(json).unwrap().celsius
    }

    #[test]
    fn numbers_are_celsius() {
        assert_eq!(celsius("21.5"), 21.5);
        assert_eq!(celsius("21"), 21.0);
        assert_eq!(celsius("-4"), -4.0);
        let from_toml: Reading = toml::from_str("outside = 12\ninside = 20.5").unwrap();
        assert_eq!(from_toml.outside.celsius, 12.0);
    }

    #[test]
    fn strings_have_a_unit() {
        assert_eq!(celsius(r#""21.5C""#), 21.5);
        assert_eq!(celsius(r#""212F""#), 100.0);
        assert_eq!(celsius(r#""-40F""#), -40.0);
    }

    #[test]
    fn errors_say_what_was_expected() {
        let err = serde_json::from_str::<Temperature>("true").unwrap_err();
        assert_eq!(
            err.to_string(),
            "invalid type: boolean `true`, expected a temperature in Celsius, \
             or a string like \"21.5C\" or \"70F\" at line 1 column 4"
        );
        for bad in [r#""hot""#, r#""21K""#, r#""C""#, r#""""#] {
            let err = serde_json::from_str::<Temperature>(bad).unwrap_err();
            assert!(
                err.to_string().starts_with("invalid value: string"),
                "{bad}: {err}"
            );
        }
    }
}
