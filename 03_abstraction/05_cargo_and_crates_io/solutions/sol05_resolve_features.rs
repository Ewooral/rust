//! Solution to exercise 5: switch features on the way Cargo does.
//!
//! A to-do list of names. Each name taken off the list is either a `dep:`
//! (recorded as a dependency) or a feature: if it was not on yet, switch it
//! on and add everything it lists to the to-do list. Features already on
//! are skipped, so nothing is done twice (and a loop in the table could not
//! run forever).
//!
//! Test: cargo test -p cargo_and_crates_io --features solutions --bin sol05_resolve_features

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
    let mut resolved = Resolved::default();
    let mut todo: Vec<&str> = requested.to_vec();
    if default_features && table.iter().any(|(name, _)| *name == "default") {
        todo.push("default");
    }
    while let Some(name) = todo.pop() {
        if let Some(dependency) = name.strip_prefix("dep:") {
            resolved.dependencies.insert(dependency.to_string());
            continue;
        }
        if resolved.features.contains(name) {
            continue; // already on
        }
        let (_, switches_on) = table
            .iter()
            .find(|(feature, _)| *feature == name)
            .ok_or_else(|| name.to_string())?;
        resolved.features.insert(name.to_string());
        todo.extend(switches_on.iter().copied());
    }
    Ok(resolved)
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
