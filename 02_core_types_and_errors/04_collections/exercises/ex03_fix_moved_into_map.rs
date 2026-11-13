//! Exercise 3: make it compile (E0382), WITHOUT `.clone()`.
//!
//! `insert` took ownership of `name`. Find two different fixes:
//!   a) change the ORDER of the lines, or
//!   b) read the name back out of the map instead of from `name`.
//!
//! Run: cargo run -p collections --features exercises --bin ex03_fix_moved_into_map

use std::collections::HashMap;

fn main() {
    let mut ages = HashMap::new();
    let name = String::from("Ama");
    ages.insert(name, 9);
    println!("added {name}, the map now has {} entries", ages.len());
}
