//! Exercise 5: switch features on the way Cargo does (Beyond the Book).
//!
//! A [features] table says what each feature switches on:
//!
//!     default = ["unicode"]
//!     unicode = []
//!     json    = ["dep:serde", "dep:serde_json"]
//!     full    = ["json", "unicode"]
//!
//! - Switching a feature on also switches on everything in its list, and
//!   everything in THEIR lists, and so on.
//! - `dep:serde` means "the optional dependency serde". Collect those in
//!   `dependencies`, without the `dep:` prefix.
//! - `default` is switched on as well, unless `default_features` is false
//!   (`--no-default-features`), or the table has no `default` line.
//! - Asking for a feature that is not in the table is an error: return
//!   `Err` with its name.
//!
//! Implement `resolve`. The tests are written; make them pass.
//! Hint: keep a to-do list (a Vec) of names still to switch on.
//!
//! Test: cargo test -p cargo_and_crates_io --features exercises --bin ex05_resolve_features

#![allow(unused_variables)]

use std::collections::BTreeSet;

/// A [features] table: each feature's name and what it switches on.
type FeatureTable<'a> = &'a [(&'a str, &'a [&'a str])];

/// Everything that ends up switched on. BTreeSets keep it sorted.
#[derive(Debug, Default, PartialEq)]
struct Resolved {
    features: BTreeSet<String>,
    dependencies: BTreeSet<String>,
}

fn resolve(
    table: FeatureTable,
    requested: &[&str],
    default_features: bool,
) -> Result<Resolved, String> {
    todo!()
}

const TABLE: FeatureTable = &[
    ("default", &["unicode"]),
    ("unicode", &[]),
    ("json", &["dep:serde", "dep:serde_json"]),
    ("full", &["json", "unicode"]),
];

fn main() {
    for (requested, defaults) in [
        (&[][..], true),
        (&["json"][..], false),
        (&["full"][..], true),
    ] {
        match resolve(TABLE, requested, defaults) {
            Ok(r) => println!(
                "{requested:?} (defaults {defaults}): features {:?}, deps {:?}",
                r.features, r.dependencies
            ),
            Err(name) => println!("unknown feature `{name}`"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|n| n.to_string()).collect()
    }

    #[test]
    fn default_is_on_unless_switched_off() {
        let r = resolve(TABLE, &[], true).unwrap();
        assert_eq!(r.features, set(&["default", "unicode"]));
        assert!(r.dependencies.is_empty());

        let r = resolve(TABLE, &[], false).unwrap();
        assert_eq!(r, Resolved::default());
    }

    #[test]
    fn features_switch_on_features_and_optional_dependencies() {
        let r = resolve(TABLE, &["full"], false).unwrap();
        assert_eq!(r.features, set(&["full", "json", "unicode"]));
        assert_eq!(r.dependencies, set(&["serde", "serde_json"]));

        let twice = resolve(TABLE, &["json", "json"], false).unwrap();
        assert_eq!(
            twice.features,
            set(&["json"]),
            "asking twice changes nothing"
        );
    }

    #[test]
    fn unknown_features_are_errors() {
        assert_eq!(resolve(TABLE, &["jsno"], true), Err("jsno".to_string()));
        let broken: FeatureTable = &[("a", &["b"])];
        assert_eq!(resolve(broken, &["a"], true), Err("b".to_string()));
        // A table without a `default` line is fine with default features on.
        assert!(resolve(broken, &[], true).unwrap().features.is_empty());
    }
}
