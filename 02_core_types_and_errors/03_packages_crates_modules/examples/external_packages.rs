//! Using external packages from crates.io, and the standard library.
//! Run: cargo run -p packages_crates_modules --example external_packages
//!
//! Step 1: list it in Cargo.toml. Here it is under [dev-dependencies]
//!         because only this example needs it:   rand = "0.9"
//! Step 2: `use` the items you need. The crate name is the first part.

use rand::Rng;

// `std` is also an external crate, but it ships with Rust, so it never
// needs a line in Cargo.toml.
use std::collections::HashMap;

fn main() {
    let mut rng = rand::rng();
    let mut rolls: HashMap<u8, u32> = HashMap::new();
    for _ in 0..600 {
        let roll = rng.random_range(1..=6); // `random_range` comes from the `Rng` trait
        *rolls.entry(roll).or_insert(0) += 1;
    }
    let mut faces: Vec<_> = rolls.into_iter().collect();
    faces.sort();
    for (face, count) in faces {
        println!("{face}: {count}");
    }
    // Remove `use rand::Rng;` and the call fails with E0599: the method
    // exists, but its trait is not in scope. Traits are lesson 03/01.
}
