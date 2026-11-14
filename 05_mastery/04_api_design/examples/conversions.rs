//! Conversion traits: `From`/`Into`, `TryFrom`/`TryInto`, `FromStr`, `AsRef`,
//! and how `?` uses `From` to convert errors.
//!
//! Run: cargo run -p api_design --example conversions

use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

use api_design::{Method, Url};

/// A percentage, 0 to 100.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Percent(u8);

/// Infallible: every `Percent` is a valid fraction. Implement `From`, and
/// `Into` comes free (`let f: f64 = p.into()`). Implementing `Into`
/// directly is discouraged; clippy says "an implementation of `From` is
/// preferred since it gives you `Into<_>` for free where the reverse isn't
/// true" (`clippy::from_over_into`).
impl From<Percent> for f64 {
    fn from(p: Percent) -> f64 {
        f64::from(p.0) / 100.0
    }
}

/// Fallible: not every `i32` is a percentage, so `TryFrom` with an error.
impl TryFrom<i32> for Percent {
    type Error = OutOfRange;

    fn try_from(n: i32) -> Result<Self, Self::Error> {
        u8::try_from(n)
            .ok()
            .filter(|n| *n <= 100)
            .map(Percent)
            .ok_or(OutOfRange(n))
    }
}

#[derive(Debug, PartialEq)]
struct OutOfRange(i32);

impl fmt::Display for OutOfRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} is not between 0 and 100", self.0)
    }
}

/// `FromStr` gives you `"42%".parse::<Percent>()`.
impl FromStr for Percent {
    type Err = ParsePercentError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let digits = text.trim().trim_end_matches('%');
        let n: i32 = digits.parse()?; // ParseIntError -> ParsePercentError, by `From` below
        Ok(Percent::try_from(n)?) // OutOfRange -> ParsePercentError, by `From` below
    }
}

#[derive(Debug)]
enum ParsePercentError {
    NotANumber(ParseIntError),
    OutOfRange(OutOfRange),
}

impl fmt::Display for ParsePercentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParsePercentError::NotANumber(e) => write!(f, "not a number ({e})"),
            ParsePercentError::OutOfRange(e) => write!(f, "{e}"),
        }
    }
}

// These two `From` impls are what lets `?` convert the errors above.
impl From<ParseIntError> for ParsePercentError {
    fn from(e: ParseIntError) -> Self {
        ParsePercentError::NotANumber(e)
    }
}

impl From<OutOfRange> for ParsePercentError {
    fn from(e: OutOfRange) -> Self {
        ParsePercentError::OutOfRange(e)
    }
}

/// `AsRef<str>` accepts `&str`, `String`, `&String`, and the library's `Url`.
fn shout(text: impl AsRef<str>) -> String {
    text.as_ref().to_uppercase()
}

fn main() {
    let half = Percent(50);
    let as_float: f64 = half.into(); // From<Percent> for f64, used as Into
    println!("Percent(50) into f64 = {as_float}");

    println!("Percent::try_from(75)  = {:?}", Percent::try_from(75));
    let too_big: Result<Percent, _> = 250.try_into(); // TryInto comes free too
    println!("250.try_into()         = {too_big:?}");
    if let Err(error) = too_big {
        println!("  as text: {error}");
    }

    for text in ["42%", "101%", "lots"] {
        match text.parse::<Percent>() {
            Ok(percent) => println!("{text:>5}.parse::<Percent>() = {percent:?}"),
            Err(error) => println!("{text:>5}.parse::<Percent>() failed: {error}"),
        }
    }

    // The library's types use the same traits:
    let url: Url = "http://example.org/a".parse().expect("FromStr");
    let same: Url = Url::try_from("http://example.org/a").expect("TryFrom<&str>");
    let method: Method = "put".parse().expect("FromStr");
    let text: String = url.clone().into(); // From<Url> for String
    println!(
        "\nurl == same: {}, method: {method}, text: {text}",
        url == same
    );
    println!(
        "{} {} {}",
        shout("str"),
        shout(String::from("string")),
        shout(&url)
    );
}
