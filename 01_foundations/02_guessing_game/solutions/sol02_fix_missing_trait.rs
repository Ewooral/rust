//! Solution to exercise 2: make it compile.
//!
//! `random_range` is a method of the `Rng` trait. A trait's methods can only
//! be called when the trait is in scope, so we bring it in with `use`.
//!
//! Run: cargo run -p guessing_game --features solutions --bin sol02_fix_missing_trait

use rand::Rng;

fn main() {
    let roll = rand::rng().random_range(1..=6);
    println!("You rolled a {roll}");
}
