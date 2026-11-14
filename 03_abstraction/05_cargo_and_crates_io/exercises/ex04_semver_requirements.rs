//! Exercise 4: read version requirements the way Cargo does (Book 14.2).
//!
//! In Cargo.toml, `rand = "0.8.5"` does NOT mean "exactly 0.8.5". It is a
//! caret requirement, `^0.8.5`: any version that semantic versioning says
//! is compatible, and at least 0.8.5.
//! - `1.2.3` accepts >= 1.2.3 and < 2.0.0   (the same MAJOR, when it is not 0)
//! - `0.2.3` accepts >= 0.2.3 and < 0.3.0   (major 0: the same MINOR)
//! - `0.0.3` accepts only 0.0.3             (major and minor 0: exact)
//!
//! Implement `Version::parse` and `accepts`. The tests are written; make
//! them pass. Hint: deriving `PartialOrd` compares the fields in order.
//!
//! Test: cargo test -p cargo_and_crates_io --features exercises --bin ex04_semver_requirements

#![allow(unused_variables, dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

impl Version {
    /// `"1.2.3"` gives `Some(Version)`. Anything that is not exactly three
    /// whole numbers separated by dots gives `None`.
    fn parse(text: &str) -> Option<Version> {
        todo!()
    }
}

/// Whether the caret requirement `requirement` accepts `candidate`.
fn accepts(requirement: Version, candidate: Version) -> bool {
    todo!()
}

fn main() {
    let requirement = Version::parse("0.8.5").expect("a valid version");
    for text in ["0.8.4", "0.8.5", "0.8.9", "0.9.0", "1.0.0"] {
        let candidate = Version::parse(text).expect("a valid version");
        let answer = if accepts(requirement, candidate) {
            "yes"
        } else {
            "no"
        };
        println!("rand = \"0.8.5\" accepts {text}? {answer}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(text: &str) -> Version {
        Version::parse(text).unwrap()
    }

    #[test]
    fn parses_exactly_three_numbers() {
        assert_eq!(
            Version::parse("10.0.42"),
            Some(Version {
                major: 10,
                minor: 0,
                patch: 42
            })
        );
        for bad in ["1.2", "1.2.3.4", "a.b.c", "", "1..3", "-1.2.3"] {
            assert_eq!(Version::parse(bad), None, "{bad:?}");
        }
        assert!(v("1.2.3") < v("1.10.0"), "numbers, not text: 2 < 10");
    }

    #[test]
    fn major_one_and_above_keeps_the_major() {
        let req = v("1.2.3");
        assert!(accepts(req, v("1.2.3")));
        assert!(accepts(req, v("1.9.0")));
        assert!(!accepts(req, v("1.2.2")), "too old");
        assert!(!accepts(req, v("2.0.0")), "a new major may break you");
    }

    #[test]
    fn major_zero_keeps_the_minor() {
        let req = v("0.2.3");
        assert!(accepts(req, v("0.2.3")));
        assert!(accepts(req, v("0.2.9")));
        assert!(!accepts(req, v("0.3.0")));
        assert!(!accepts(req, v("1.0.0")));
    }

    #[test]
    fn zero_zero_is_exact() {
        let req = v("0.0.3");
        assert!(accepts(req, v("0.0.3")));
        assert!(!accepts(req, v("0.0.4")));
    }
}
