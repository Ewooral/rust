//! Exercise 6: make `?` convert errors for you with `From`.
//!
//! `parse_config` reads text like:
//!     name = shop
//!     port = 8080
//! Blank lines are skipped. Every other line must be `key = value`.
//!
//! Your jobs:
//!   1. Write `impl From<ParseIntError> for ConfigError` so that
//!      `value.parse::<u16>()?` turns a bad port into ConfigError::BadPort.
//!   2. Implement `parse_config` using `?` (no `match` on the parse result).
//!      Errors: a line without '=' -> BadLine(line number, starting at 1);
//!      a missing key at the end -> MissingKey("name") / MissingKey("port");
//!      other keys are ignored.
//!
//! Test: cargo test -p error_handling --features exercises --bin ex06_question_mark_with_from

#![allow(unused_variables, unused_imports)]

use std::num::ParseIntError;

#[derive(Debug, PartialEq)]
struct Config {
    name: String,
    port: u16,
}

#[derive(Debug, PartialEq)]
enum ConfigError {
    BadLine(usize),
    MissingKey(&'static str),
    BadPort(ParseIntError),
}

// TODO 1: impl From<ParseIntError> for ConfigError

fn parse_config(text: &str) -> Result<Config, ConfigError> {
    todo!()
}

fn main() {
    println!("{:?}", parse_config("name = shop\nport = 8080"));
    println!("{:?}", parse_config("name = shop\nport = eighty"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_good_config() {
        let text = "\n name = shop \n\n colour = red\nport=8080\n";
        assert_eq!(
            parse_config(text),
            Ok(Config {
                name: "shop".into(),
                port: 8080
            })
        );
    }

    #[test]
    fn reports_bad_lines_and_missing_keys() {
        assert_eq!(
            parse_config("name = a\nthis line is wrong"),
            Err(ConfigError::BadLine(2))
        );
        assert_eq!(
            parse_config("port = 1"),
            Err(ConfigError::MissingKey("name"))
        );
        assert_eq!(
            parse_config("name = a"),
            Err(ConfigError::MissingKey("port"))
        );
    }

    #[test]
    fn bad_port_keeps_the_parse_error() {
        match parse_config("name = a\nport = 99999") {
            Err(ConfigError::BadPort(e)) => {
                assert_eq!(e.to_string(), "number too large to fit in target type")
            }
            other => panic!("expected BadPort, got {other:?}"),
        }
    }
}
