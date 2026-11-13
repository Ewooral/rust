//! Beyond the Book: what `#[derive(...)]` gives you for free.
//! Run: cargo run -p structs --example derives

use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
struct Point {
    x: i32,
    y: i32,
}

// A struct holding a String can be Clone but never Copy.
#[derive(Debug, Clone, PartialEq, Default)]
struct Settings {
    name: String,
    volume: u8,
    muted: bool,
}

fn main() {
    let a = Point { x: 1, y: 2 };
    let b = a; // Copy: `a` is still usable
    println!("Debug: {a:?}");
    println!("PartialEq: {}", a == b);
    println!(
        "PartialOrd (field by field, top to bottom): {}",
        a < Point { x: 1, y: 3 }
    );
    println!("Default: {:?}", Point::default());

    // Hash + Eq let a type be a HashSet / HashMap key.
    let mut seen = HashSet::new();
    seen.insert(a);
    seen.insert(b);
    println!("unique points: {}", seen.len());

    // Default + struct update syntax: set only what differs.
    let s = Settings {
        volume: 7,
        ..Default::default()
    };
    let t = s.clone();
    println!("{s:?}, equal to clone: {}", s == t);

    // Derive order of traits does not matter. What each derive needs:
    // Copy needs Clone. Eq needs PartialEq. Ord needs Eq + PartialOrd.
    // Every derive needs every FIELD to implement that trait too.
}
