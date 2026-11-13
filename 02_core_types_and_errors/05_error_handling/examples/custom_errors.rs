//! Beyond the Book: your own error type, written by hand.
//! Run: cargo run -p error_handling --example custom_errors
//!
//! A good error type: an enum with one variant per kind of failure, that
//! implements Display (for people), Debug (for programmers) and Error (so it
//! works with `?`, Box<dyn Error> and other libraries), and From (so `?` can
//! convert lower-level errors into it automatically).

use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
enum ConfigError {
    MissingKey(&'static str),
    BadNumber { key: String, source: ParseIntError },
    OutOfRange { key: String, value: u32 },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingKey(key) => write!(f, "missing key `{key}`"),
            ConfigError::BadNumber { key, .. } => write!(f, "`{key}` is not a whole number"),
            ConfigError::OutOfRange { key, value } => {
                write!(f, "`{key}` = {value} is out of range")
            }
        }
    }
}

impl Error for ConfigError {
    /// The lower-level error that caused this one, if any.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::BadNumber { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// A tiny parser: "key=value" pairs separated by commas.
fn lookup<'a>(text: &'a str, key: &'static str) -> Result<&'a str, ConfigError> {
    text.split(',')
        .filter_map(|pair| pair.split_once('='))
        .find(|(k, _)| k.trim() == key)
        .map(|(_, v)| v.trim())
        .ok_or(ConfigError::MissingKey(key))
}

fn read_threads(text: &str) -> Result<u32, ConfigError> {
    let raw = lookup(text, "threads")?;
    // map_err converts one error type into another by hand.
    let value: u32 = raw.parse().map_err(|e| ConfigError::BadNumber {
        key: "threads".into(),
        source: e,
    })?;
    if !(1..=64).contains(&value) {
        return Err(ConfigError::OutOfRange {
            key: "threads".into(),
            value,
        });
    }
    Ok(value)
}

// ---- `?` with automatic From conversion ----

#[derive(Debug)]
enum AppError {
    Config(ConfigError),
    Parse(ParseIntError),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Config(e) => write!(f, "configuration problem: {e}"),
            AppError::Parse(e) => write!(f, "parse problem: {e}"),
        }
    }
}

impl Error for AppError {}

// `?` calls `From::from` on the error, so these impls let it convert for us.
impl From<ConfigError> for AppError {
    fn from(e: ConfigError) -> Self {
        AppError::Config(e)
    }
}
impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self {
        AppError::Parse(e)
    }
}

fn start(config: &str, retries: &str) -> Result<String, AppError> {
    let threads = read_threads(config)?; // ConfigError -> AppError
    let retries: u32 = retries.parse()?; // ParseIntError -> AppError
    Ok(format!("{threads} threads, {retries} retries"))
}

fn main() {
    for config in ["threads=8", "colour=blue", "threads=eight", "threads=500"] {
        match read_threads(config) {
            Ok(n) => println!("{config:>14} -> {n} threads"),
            Err(e) => {
                print!("{config:>14} -> error: {e}");
                if let Some(cause) = e.source() {
                    print!(" (caused by: {cause})");
                }
                println!();
            }
        }
    }

    println!("{:?}", start("threads=4", "3"));
    match start("threads=4", "three") {
        Ok(s) => println!("{s}"),
        Err(e) => println!("{e}"),
    }
}
