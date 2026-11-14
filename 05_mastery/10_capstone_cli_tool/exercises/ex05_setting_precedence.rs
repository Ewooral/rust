//! Exercise 5: add a new setting, `max_width` (how many characters of each
//! message the table shows), through all four layers, the way logsum's
//! src/config.rs does for `top`:
//!
//!   built-in default (100)  <  config file  <  LOGSUM_MAX_WIDTH  <  --max-width
//!
//! Implement:
//! - `from_file`: parse config-file text. `max_width = 60` gives `Some(60)`,
//!   no key gives `None`, and unknown keys or a wrong type are an `Err`
//!   (the `FileLayer` struct is ready: `toml::from_str` does the work).
//! - `from_env`: read `LOGSUM_MAX_WIDTH` through `get` (never the real
//!   environment, so tests can't interfere with each other). Unset or empty
//!   gives `None`; a bad value gives
//!   `Err("invalid LOGSUM_MAX_WIDTH: \"wide\"")`.
//! - `resolve`: the winning value and where it came from.
//! - `validate`: a width below 10 makes no sense. The error must say where
//!   the bad value came from, or the user can't find it:
//!   `"max_width = 3 (from environment) is too small: the minimum is 10"`.
//!
//! Test: cargo test -p capstone_cli_tool --features exercises --bin ex05_setting_precedence

#![allow(unused_variables, dead_code)]

use std::collections::HashMap;
use std::fmt;

use serde::Deserialize;

const DEFAULT_MAX_WIDTH: usize = 100;
const MIN_WIDTH: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Source {
    Default,
    File,
    Env,
    Flag,
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Source::Default => "default",
            Source::File => "config file",
            Source::Env => "environment",
            Source::Flag => "flag",
        })
    }
}

/// The config file's shape, for this one setting.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileLayer {
    max_width: Option<usize>,
}

fn from_file(text: &str) -> Result<Option<usize>, String> {
    todo!()
}

fn from_env(get: impl Fn(&str) -> Option<String>) -> Result<Option<usize>, String> {
    todo!()
}

fn resolve(file: Option<usize>, env: Option<usize>, flag: Option<usize>) -> (usize, Source) {
    todo!()
}

fn validate(value: usize, source: Source) -> Result<usize, String> {
    todo!()
}

fn main() {
    let fake_env = HashMap::from([("LOGSUM_MAX_WIDTH".to_string(), "60".to_string())]);
    let file = from_file("max_width = 80\n").unwrap();
    let env = from_env(|key| fake_env.get(key).cloned()).unwrap();
    let (value, source) = resolve(file, env, None);
    println!("max_width = {value} (from {source})");
    println!("{:?}", validate(value, source));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |key| map.get(key).cloned()
    }

    #[test]
    fn file_layer() {
        assert_eq!(from_file("max_width = 60\n"), Ok(Some(60)));
        assert_eq!(from_file("# nothing set\n"), Ok(None));
        assert!(from_file("max_wdith = 60\n").is_err(), "typo");
        assert!(from_file("max_width = \"wide\"\n").is_err(), "wrong type");
    }

    #[test]
    fn env_layer() {
        assert_eq!(from_env(env(&[("LOGSUM_MAX_WIDTH", "42")])), Ok(Some(42)));
        assert_eq!(from_env(env(&[])), Ok(None));
        assert_eq!(from_env(env(&[("LOGSUM_MAX_WIDTH", "")])), Ok(None));
        assert_eq!(
            from_env(env(&[("LOGSUM_MAX_WIDTH", "wide")])),
            Err("invalid LOGSUM_MAX_WIDTH: \"wide\"".to_string())
        );
    }

    #[test]
    fn later_layers_win() {
        assert_eq!(
            resolve(None, None, None),
            (DEFAULT_MAX_WIDTH, Source::Default)
        );
        assert_eq!(resolve(Some(80), None, None), (80, Source::File));
        assert_eq!(resolve(Some(80), Some(60), None), (60, Source::Env));
        assert_eq!(resolve(Some(80), Some(60), Some(40)), (40, Source::Flag));
        assert_eq!(resolve(None, None, Some(40)), (40, Source::Flag));
    }

    #[test]
    fn validation_names_the_source() {
        assert_eq!(validate(10, Source::Flag), Ok(10));
        assert_eq!(
            validate(3, Source::Env),
            Err("max_width = 3 (from environment) is too small: the minimum is 10".to_string())
        );
        assert_eq!(
            validate(0, Source::File),
            Err("max_width = 0 (from config file) is too small: the minimum is 10".to_string())
        );
    }
}
