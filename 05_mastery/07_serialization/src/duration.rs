//! `Duration` as friendly text: `"30s"`, `"5m"`, `"1h30m"`, `"250ms"`.
//!
//! Serde has no opinion about how a `Duration` should look in a file (its
//! built-in impl writes `{"secs":30,"nanos":0}`, which no human wants to
//! type). This module is a **`with` module**: put
//! `#[serde(with = "crate::duration")]` on a field and Serde calls our
//! `serialize` and `deserialize` functions for that one field instead.
//!
//! Reading uses a hand-written [`Visitor`], so it can accept two shapes:
//! a string (`"1h30m"`) or a plain number of seconds (`90`).

use std::fmt;
use std::time::Duration;

use serde::de::{self, Visitor};
use serde::{Deserializer, Serializer};

/// Writes the duration as text, the shortest way: `5400s` becomes `"1h30m"`.
pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format_duration(*duration))
}

/// Reads `"30s"`, `"1h30m"`, `"250ms"`, or a whole number of seconds.
pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    // `deserialize_any` means "tell me what you've got": the format looks at
    // the input and calls visit_str or visit_u64/visit_i64. Only formats that
    // describe themselves (JSON, TOML) can do this; postcard cannot.
    deserializer.deserialize_any(DurationVisitor)
}

/// The Visitor: one method per kind of value we are willing to accept.
struct DurationVisitor;

impl Visitor<'_> for DurationVisitor {
    type Value = Duration;

    // Used in error messages: "invalid type: boolean `true`, expected ...".
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a duration such as \"30s\", \"5m\", \"1h30m\" or a number of seconds")
    }

    // JSON gives non-negative integers as u64...
    fn visit_u64<E: de::Error>(self, secs: u64) -> Result<Duration, E> {
        Ok(Duration::from_secs(secs))
    }

    // ...but TOML integers are always i64, so we need this one too.
    fn visit_i64<E: de::Error>(self, secs: i64) -> Result<Duration, E> {
        u64::try_from(secs)
            .map(Duration::from_secs)
            .map_err(|_| E::invalid_value(de::Unexpected::Signed(secs), &self))
    }

    fn visit_str<E: de::Error>(self, text: &str) -> Result<Duration, E> {
        parse_duration(text).map_err(|why| E::custom(format!("bad duration {text:?}: {why}")))
    }
}

/// Parses `"1h30m"`, `"45s"`, `"250ms"`, `"2h"`. Units: `h`, `m`, `s`, `ms`.
///
/// # Errors
///
/// A short reason when the text is empty, a number has no unit, the unit is
/// unknown, or the value is too large.
///
/// # Examples
///
/// ```
/// use std::time::Duration;
/// use serialization::duration::parse_duration;
///
/// assert_eq!(parse_duration("1h30m"), Ok(Duration::from_secs(5400)));
/// assert!(parse_duration("10 parsecs").is_err());
/// ```
pub fn parse_duration(text: &str) -> Result<Duration, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("it is empty".to_string());
    }
    let mut total = Duration::ZERO;
    let mut rest = text;
    while !rest.is_empty() {
        let digits = rest.chars().take_while(char::is_ascii_digit).count();
        if digits == 0 {
            return Err(format!("expected a number at {rest:?}"));
        }
        let number: u64 = rest[..digits]
            .parse()
            .map_err(|_| "the number is too large".to_string())?;
        rest = &rest[digits..];
        let unit_len = rest.chars().take_while(char::is_ascii_alphabetic).count();
        let part = match &rest[..unit_len] {
            "h" => number.checked_mul(3600).map(Duration::from_secs),
            "m" => number.checked_mul(60).map(Duration::from_secs),
            "s" => Some(Duration::from_secs(number)),
            "ms" => Some(Duration::from_millis(number)),
            "" => return Err(format!("{number} needs a unit: h, m, s or ms")),
            other => return Err(format!("unknown unit {other:?}: use h, m, s or ms")),
        };
        total = part
            .and_then(|part| total.checked_add(part))
            .ok_or("the duration is too large")?;
        rest = &rest[unit_len..];
    }
    Ok(total)
}

/// The reverse of [`parse_duration`]: `Duration::from_secs(5400)` is `"1h30m"`.
///
/// Anything below a millisecond is dropped.
pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();
    let (hours, minutes, seconds) = (secs / 3600, secs / 60 % 60, secs % 60);
    let mut out = String::new();
    for (value, unit) in [
        (hours, "h"),
        (minutes, "m"),
        (seconds, "s"),
        (u64::from(millis), "ms"),
    ] {
        if value > 0 {
            out.push_str(&format!("{value}{unit}"));
        }
    }
    if out.is_empty() {
        out.push_str("0s");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Timer {
        #[serde(with = "crate::duration")]
        every: Duration,
    }

    #[test]
    fn parses_units_and_combinations() {
        assert_eq!(parse_duration("45s"), Ok(Duration::from_secs(45)));
        assert_eq!(parse_duration("2h"), Ok(Duration::from_secs(7200)));
        assert_eq!(parse_duration("1h30m"), Ok(Duration::from_secs(5400)));
        assert_eq!(parse_duration("250ms"), Ok(Duration::from_millis(250)));
        assert_eq!(parse_duration(" 1m5s "), Ok(Duration::from_secs(65)));
    }

    #[test]
    fn rejects_nonsense() {
        assert!(parse_duration("").unwrap_err().contains("empty"));
        assert!(parse_duration("30").unwrap_err().contains("needs a unit"));
        assert!(parse_duration("3 days").is_err());
        assert!(parse_duration("3d").unwrap_err().contains("unknown unit"));
        assert!(parse_duration("99999999999999999999s").is_err());
    }

    #[test]
    fn formats_the_shortest_way() {
        assert_eq!(format_duration(Duration::from_secs(5400)), "1h30m");
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(Duration::from_millis(1250)), "1s250ms");
        assert_eq!(format_duration(Duration::ZERO), "0s");
    }

    #[test]
    fn round_trips_through_json_and_toml() {
        let timer = Timer {
            every: Duration::from_secs(90),
        };
        let json = serde_json::to_string(&timer).unwrap();
        assert_eq!(json, r#"{"every":"1m30s"}"#);
        assert_eq!(serde_json::from_str::<Timer>(&json).unwrap(), timer);
        let toml_text = toml::to_string(&timer).unwrap();
        assert_eq!(toml_text, "every = \"1m30s\"\n");
        assert_eq!(toml::from_str::<Timer>(&toml_text).unwrap(), timer);
    }

    #[test]
    fn accepts_plain_seconds_from_json_u64_and_toml_i64() {
        let from_json: Timer = serde_json::from_str(r#"{"every": 90}"#).unwrap();
        let from_toml: Timer = toml::from_str("every = 90").unwrap();
        assert_eq!(from_json.every, Duration::from_secs(90));
        assert_eq!(from_toml.every, Duration::from_secs(90));
    }

    #[test]
    fn errors_say_what_was_expected() {
        let err = serde_json::from_str::<Timer>(r#"{"every": true}"#).unwrap_err();
        assert!(
            err.to_string()
                .starts_with("invalid type: boolean `true`, expected a duration such as \"30s\"")
        );
        let err = serde_json::from_str::<Timer>(r#"{"every": -5}"#).unwrap_err();
        assert!(err.to_string().starts_with("invalid value: integer `-5`"));
    }
}
