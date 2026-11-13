//! Exercise 6 (Beyond the Book): implement std traits for your own type.
//!
//! An `Rgb` colour should:
//!   - be created from a tuple with `From<(u8, u8, u8)>` (so `.into()` works too),
//!   - be parsed from text like "#ff8800" with `TryFrom<&str>`; anything that
//!     is not '#' followed by exactly 6 hex digits is an error,
//!   - print as "#ff8800" with `Display` (lowercase, 2 digits each).
//!
//! Hint: `u8::from_str_radix("ff", 16)` and `format!("{:02x}", n)`.
//!
//! Test: cargo test -p generics_traits_lifetimes --features exercises --bin ex06_conversion_traits

#![allow(unused_variables, dead_code)]

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Debug, PartialEq)]
struct ParseColorError(String);

// TODO: impl From<(u8, u8, u8)> for Rgb

// TODO: impl TryFrom<&str> for Rgb  (type Error = ParseColorError)

// TODO: impl fmt::Display for Rgb

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_and_into() {
        assert_eq!(
            Rgb::from((255, 136, 0)),
            Rgb {
                r: 255,
                g: 136,
                b: 0
            }
        );
        let c: Rgb = (1, 2, 3).into();
        assert_eq!(c.b, 3);
    }

    #[test]
    fn try_from_text() {
        assert_eq!(
            Rgb::try_from("#FF8800"),
            Ok(Rgb {
                r: 255,
                g: 136,
                b: 0
            })
        );
        let parsed: Result<Rgb, _> = "#000000".try_into();
        assert_eq!(parsed, Ok(Rgb { r: 0, g: 0, b: 0 }));
        for bad in ["ff8800", "#ff88", "#gg8800", "#ff88001", ""] {
            assert!(Rgb::try_from(bad).is_err(), "{bad:?} should fail");
        }
    }

    #[test]
    fn displays_as_hex() {
        assert_eq!(Rgb::from((255, 136, 0)).to_string(), "#ff8800");
        assert_eq!(Rgb::from((0, 10, 1)).to_string(), "#000a01");
    }
}
