//! Exercise 6: which settings does a profile really use? (Book 14.1)
//!
//! Cargo has built-in defaults for its two main profiles (`DEV` and
//! `RELEASE` below). A `[profile.NAME]` table in the workspace's Cargo.toml
//! overrides ONLY the settings it writes. A custom profile (any other name)
//! must say which profile it starts from, with `inherits = "release"` (or
//! "dev", or another custom profile), and then overrides on top of that.
//!
//!     [profile.dev]
//!     opt-level = 1                # dev, but a little faster
//!
//!     [profile.release-with-debug]
//!     inherits = "release"
//!     debug = true                 # release, plus debug info for a profiler
//!
//! Implement `effective`. Errors (as `Err` with a message):
//! - a custom profile with no table: the message contains "no profile",
//! - a custom profile without `inherits`: the message contains "inherits".
//!
//! The tests are written; make them pass.
//!
//! Test: cargo test -p cargo_and_crates_io --features exercises --bin ex06_profile_inheritance

#![allow(unused_variables, dead_code)]

/// The settings that matter for this exercise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Settings {
    opt_level: u8,
    debug: bool,
    overflow_checks: bool,
}

const DEV: Settings = Settings {
    opt_level: 0,
    debug: true,
    overflow_checks: true,
};

const RELEASE: Settings = Settings {
    opt_level: 3,
    debug: false,
    overflow_checks: false,
};

/// One `[profile.NAME]` table. `None` means "not written in the table".
#[derive(Debug, Default, Clone)]
struct ProfileTable {
    name: &'static str,
    inherits: Option<&'static str>,
    opt_level: Option<u8>,
    debug: Option<bool>,
    overflow_checks: Option<bool>,
}

/// The settings `cargo build --profile <name>` would use.
fn effective(name: &str, tables: &[ProfileTable]) -> Result<Settings, String> {
    todo!()
}

fn main() {
    let tables = [ProfileTable {
        name: "release-with-debug",
        inherits: Some("release"),
        debug: Some(true),
        ..ProfileTable::default()
    }];
    for name in ["dev", "release", "release-with-debug", "turbo"] {
        println!("{name:<20} {:?}", effective(name, &tables));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_profiles_use_their_defaults() {
        assert_eq!(effective("dev", &[]), Ok(DEV));
        assert_eq!(effective("release", &[]), Ok(RELEASE));
    }

    #[test]
    fn a_table_overrides_only_what_it_writes() {
        let tables = [ProfileTable {
            name: "dev",
            opt_level: Some(1),
            ..ProfileTable::default()
        }];
        let expected = Settings {
            opt_level: 1,
            ..DEV
        };
        assert_eq!(effective("dev", &tables), Ok(expected));
    }

    #[test]
    fn custom_profiles_inherit_then_override() {
        let tables = [
            ProfileTable {
                name: "release",
                overflow_checks: Some(true),
                ..ProfileTable::default()
            },
            ProfileTable {
                name: "release-with-debug",
                inherits: Some("release"),
                debug: Some(true),
                ..ProfileTable::default()
            },
            ProfileTable {
                name: "ci",
                inherits: Some("release-with-debug"),
                opt_level: Some(2),
                ..ProfileTable::default()
            },
        ];
        let expected = Settings {
            opt_level: 3,
            debug: true,
            overflow_checks: true, // from the changed [profile.release]
        };
        assert_eq!(effective("release-with-debug", &tables), Ok(expected));
        let ci = effective("ci", &tables).unwrap();
        assert_eq!(ci.opt_level, 2);
        assert!(ci.debug, "a chain of inherits works too");
    }

    #[test]
    fn custom_profiles_need_a_table_and_inherits() {
        let missing = effective("turbo", &[]).unwrap_err();
        assert!(missing.contains("no profile"), "{missing}");
        let tables = [ProfileTable {
            name: "turbo",
            opt_level: Some(3),
            ..ProfileTable::default()
        }];
        let no_parent = effective("turbo", &tables).unwrap_err();
        assert!(no_parent.contains("inherits"), "{no_parent}");
    }
}
