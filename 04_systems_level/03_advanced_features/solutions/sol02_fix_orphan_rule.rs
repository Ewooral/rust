//! Solution to exercise 2: make it compile (E0117).
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
//! Run: cargo run -p advanced_features --features solutions --bin sol02_fix_orphan_rule

use std::fmt;
use std::ops::Deref;

/// Our own type, so we may implement any trait for it.
struct ShoppingList(Vec<String>);

impl fmt::Display for ShoppingList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in &self.0 {
            writeln!(f, "- {item}")?;
        }
        Ok(())
    }
}

// Bonus: the wrapper now behaves like the Vec for reading (`len`, `iter`,
// indexing …). Leave out DerefMut if callers shouldn't change the list.
impl Deref for ShoppingList {
    type Target = Vec<String>;
    fn deref(&self) -> &Vec<String> {
        &self.0
    }
}

fn main() {
    let shopping = ShoppingList(vec![
        String::from("eggs"),
        String::from("flour"),
        String::from("milk"),
    ]);
    print!("{shopping}");
    println!("({} items, first is {})", shopping.len(), shopping[0]);
}
