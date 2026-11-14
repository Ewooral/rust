//! Exercise 1: make it compile (E0277).
//!
//! `Version` is a public type in a library. Its users want to print it,
//! sort it, compare it, use it as a `HashSet` key and start from a default,
//! and they CAN'T add those traits themselves: the orphan rule forbids
//! implementing a foreign trait (`Ord`) for a foreign type (`Version`).
//! So a library implements the common traits up front (C-COMMON-TRAITS).
//!
//! Fix `Version` only, not `main`:
//!   - derive Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord and
//!     Default. (The derived order compares `major`, then `minor`, then
//!     `patch`: the field order matters!)
//!   - implement `std::fmt::Display` to print `1.4.2`.
//!
//! The build shows seven E0277 errors, one per missing trait use, plus an
//! E0599 ("`Copied<…>` is not an iterator") that disappears with `Copy`.
//!
//! Run: cargo run -p api_design --features exercises --bin ex01_fix_missing_traits

use std::collections::HashSet;

/// A semantic version number.
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

fn main() {
    let mut versions = vec![
        Version {
            major: 1,
            minor: 10,
            patch: 0,
        },
        Version {
            major: 1,
            minor: 4,
            patch: 2,
        },
        Version {
            major: 0,
            minor: 9,
            patch: 7,
        },
    ];
    versions.sort();
    println!("sorted: {versions:?}");

    let newest = versions[versions.len() - 1];
    println!("newest: {newest}");

    let start: Version = Default::default();
    println!(
        "default: {start}, is it in the list? {}",
        versions.contains(&start)
    );

    let unique: HashSet<Version> = versions.iter().copied().collect();
    println!("{} unique versions", unique.len());
}
