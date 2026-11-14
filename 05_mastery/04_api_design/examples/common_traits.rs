//! The traits every public type should implement when it can, and what
//! each one makes possible for your users.
//!
//! Run: cargo run -p api_design --example common_traits

use std::collections::{BTreeSet, HashMap};
use std::fmt;

/// A version number. Deriving costs nothing to write, and each derive is a
/// promise users can build on. (Adding one later is fine; REMOVING one is a
/// breaking change.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

/// `Display` is for USERS ("1.4.2"); `Debug` is for programmers
/// (`Version { major: 1, … }`). Only implement `Display` when there's one
/// obvious way to show the value as text.
impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

fn v(major: u32, minor: u32, patch: u32) -> Version {
    Version {
        major,
        minor,
        patch,
    }
}

fn main() {
    let current = v(1, 4, 2);

    // Debug: {:?} and {:#?}, assert_eq! messages, dbg!.
    println!("Debug:   {current:?}");
    // Display: {} and .to_string().
    println!("Display: {current}");
    // Clone/Copy: use a value twice. Copy only for small, plain data.
    let copy = current;
    println!("Copy:    still have both, {current} and {copy}");
    // PartialEq/Eq: == and assert_eq!.
    println!("Eq:      {}", current == v(1, 4, 2));
    // PartialOrd/Ord: <, sort(), max(), BTreeSet/BTreeMap keys. The derive
    // compares fields in order: major, then minor, then patch.
    let mut versions = vec![v(2, 0, 0), v(1, 10, 0), v(1, 4, 2)];
    versions.sort();
    println!(
        "Ord:     sorted {versions:?}, newest {}",
        versions.iter().max().unwrap()
    );
    let unique: BTreeSet<Version> = [v(1, 0, 0), v(1, 0, 0), v(0, 9, 0)].into();
    println!("         BTreeSet keeps {} unique versions", unique.len());
    // Hash (with Eq): HashMap/HashSet keys.
    let mut downloads: HashMap<Version, u32> = HashMap::new();
    *downloads.entry(current).or_default() += 1;
    println!("Hash:    downloads of {current}: {}", downloads[&current]);
    // Default: Version::default(), ..Default::default(), unwrap_or_default.
    let start = Version {
        major: 1,
        ..Default::default()
    };
    println!("Default: {}", start);

    // Send + Sync come automatically when every field has them. A test can
    // check that they stay (the library's `public_types_are_send_and_sync`):
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Version>();
    assert_send_sync::<api_design::Request>();
    println!("Send + Sync: checked at compile time");

    // Without the derives, each use above is a compile error, for example:
    //   println!("{current:?}") -> E0277: `Version` doesn't implement `Debug`
    //   versions.sort()         -> E0277: the trait bound `Version: Ord` is not satisfied
    //   Version::default()      -> E0599: no associated function or constant named
    //                              `default` found for struct `Version`
    // A user can't add them for you: the orphan rule (lesson 03/01) forbids
    // implementing a foreign trait for a foreign type.
}
