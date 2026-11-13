//! Solution to exercise 3: make it compile (E0382), WITHOUT `.clone()`.
//!
//! `insert` took ownership of `name`. Find two different fixes:
//!   a) change the ORDER of the lines, or
//!   b) read the name back out of the map instead of from `name`.
//!
//! Run: cargo run -p collections --features solutions --bin sol03_fix_moved_into_map

use std::collections::HashMap;

fn main() {
    let mut ages = HashMap::new();
    let name = String::from("Ama");
    // a) Use `name` before handing it to the map.
    println!("adding {name}");
    ages.insert(name, 9);
    println!("the map now has {} entries", ages.len());

    // b) Or read the key back from the map, which now owns it.
    for (stored_name, age) in &ages {
        println!("{stored_name} is {age}");
    }
}
