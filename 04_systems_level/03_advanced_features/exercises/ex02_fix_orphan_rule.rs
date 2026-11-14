//! Exercise 2: make it compile (E0117).
//!
//! We want `println!("{shopping}")` to print a shopping list, one item per
//! line. But `Display` is from std and `Vec` is from std, so the ORPHAN RULE
//! forbids this impl in our crate.
//!
//! Fix it with the NEWTYPE pattern: wrap the Vec in a struct of your own
//! (`struct ShoppingList(Vec<String>);`), implement `Display` for that, and
//! change `main` to use it. The output must stay the same.
//!
//! Bonus: implement `Deref` for your wrapper so `shopping.len()` works
//! without writing `.0`.
//!
//! Run: cargo run -p advanced_features --features exercises --bin ex02_fix_orphan_rule

use std::fmt;

impl fmt::Display for Vec<String> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in self {
            writeln!(f, "- {item}")?;
        }
        Ok(())
    }
}

fn main() {
    let shopping = vec![
        String::from("eggs"),
        String::from("flour"),
        String::from("milk"),
    ];
    print!("{shopping}");
}
