//! Solution to exercise 4: value parsers: turn text into checked, typed values.
//!
//! Implement the two parser functions, and tighten one built-in parser:
//!
//! - `parse_duration`: one or more `<number><unit>` pairs with the units
//!   `h`, `m` and `s`: "45s", "15m", "1h30m", "2m5s". Return an error
//!   (`Err(String)`) for an empty string, a number without a unit ("90"),
//!   a unit without a number ("m"), an unknown unit ("3d"), or a total of
//!   zero ("0s"). Hint: walk the characters, collecting digits into a
//!   number until you meet a unit letter.
//! - `parse_key_value`: "name=value" -> `("name", "value")`. Split at the
//!   FIRST `=` (the value may contain more). No `=`, or an empty name, is
//!   an error. Used by the repeatable `--set` option.
//! - `--port` must be 1024 or more: change its `value_parser` to
//!   `clap::value_parser!(u16).range(1024..)`.
//!
//! Clap shows your error text after its own "invalid value ..." message,
//! and the error kind is `ErrorKind::ValueValidation`.
//!
//! Test: cargo test -p clap_cli --features solutions --bin sol04_value_parsers

use clap::Parser;
use std::time::Duration;

/// Starts a timer (pretend)
#[derive(Parser, Debug)]
#[command(name = "timer")]
struct Cli {
    /// How long: 45s, 15m, 1h30m...
    #[arg(long, value_parser = parse_duration)]
    duration: Duration,

    /// Extra settings, repeatable: --set colour=red --set sound=bell
    #[arg(long = "set", value_name = "KEY=VALUE", value_parser = parse_key_value)]
    settings: Vec<(String, String)>,

    /// Port for the status page (1024 or more)
    #[arg(long, default_value_t = 8080, value_parser = clap::value_parser!(u16).range(1024..))]
    port: u16,
}

fn parse_duration(s: &str) -> Result<Duration, String> {
    if s.is_empty() {
        return Err("give a duration such as 45s, 15m or 1h30m".to_string());
    }
    let mut total = 0u64;
    let mut number = String::new();
    for c in s.chars() {
        if c.is_ascii_digit() {
            number.push(c);
            continue;
        }
        let seconds_per = match c {
            'h' => 3600,
            'm' => 60,
            's' => 1,
            _ => return Err(format!("unknown unit {c:?}: use h, m or s")),
        };
        let n: u64 = number
            .parse()
            .map_err(|_| format!("a number must come before {c:?}"))?;
        total += n * seconds_per;
        number.clear();
    }
    if !number.is_empty() {
        return Err(format!("{number} needs a unit: h, m or s"));
    }
    if total == 0 {
        return Err("the duration must be more than 0".to_string());
    }
    Ok(Duration::from_secs(total))
}

fn parse_key_value(s: &str) -> Result<(String, String), String> {
    match s.split_once('=') {
        Some(("", _)) => Err("the name before = is empty".to_string()),
        Some((key, value)) => Ok((key.to_string(), value.to_string())),
        None => Err(format!("expected KEY=VALUE, got {s:?}")),
    }
}

fn main() {
    let cli = Cli::parse();
    println!("{cli:?}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;

    #[test]
    fn durations_parse_and_bad_ones_are_rejected() {
        assert_eq!(parse_duration("45s"), Ok(Duration::from_secs(45)));
        assert_eq!(parse_duration("15m"), Ok(Duration::from_secs(900)));
        assert_eq!(parse_duration("1h30m"), Ok(Duration::from_secs(5400)));
        assert_eq!(parse_duration("2m5s"), Ok(Duration::from_secs(125)));
        for bad in ["", "90", "m", "3d", "0s", "1h30"] {
            assert!(parse_duration(bad).is_err(), "{bad:?} should be rejected");
        }
    }

    #[test]
    fn key_values_split_at_the_first_equals() {
        assert_eq!(
            parse_key_value("colour=red"),
            Ok(("colour".into(), "red".into()))
        );
        assert_eq!(
            parse_key_value("expr=a=b"),
            Ok(("expr".into(), "a=b".into()))
        );
        assert_eq!(
            parse_key_value("empty="),
            Ok(("empty".into(), String::new()))
        );
        assert!(parse_key_value("no-equals").is_err());
        assert!(parse_key_value("=value").is_err());
    }

    #[test]
    fn clap_uses_the_parsers() {
        let cli = Cli::try_parse_from([
            "timer",
            "--duration",
            "1h30m",
            "--set",
            "a=1",
            "--set",
            "b=2",
        ])
        .unwrap();
        assert_eq!(cli.duration, Duration::from_secs(5400));
        assert_eq!(
            cli.settings,
            [("a".into(), "1".into()), ("b".into(), "2".into())]
        );
        assert_eq!(cli.port, 8080);

        let err = Cli::try_parse_from(["timer", "--duration", "90"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::ValueValidation);
        assert!(
            err.to_string()
                .contains("invalid value '90' for '--duration <DURATION>'")
        );

        let err = Cli::try_parse_from(["timer", "--duration", "1m", "--port", "80"]).unwrap_err();
        assert_eq!(
            err.kind(),
            ErrorKind::ValueValidation,
            "ports below 1024 are refused"
        );
    }
}
