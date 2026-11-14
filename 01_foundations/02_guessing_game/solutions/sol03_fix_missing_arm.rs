//! Solution to exercise 3: make it compile.
//!
//! `Ordering` has three variants, so the `match` needs three arms (or a `_`).
//! Naming `Equal` is better than `_`: if a variant were ever added, the
//! compiler would point us here again.
//!
//! Run: cargo run -p guessing_game --features solutions --bin sol03_fix_missing_arm

use std::cmp::Ordering;

fn main() {
    let secret_number: u32 = 50;
    let guess: u32 = 50;

    match guess.cmp(&secret_number) {
        Ordering::Less => println!("Too small!"),
        Ordering::Greater => println!("Too big!"),
        Ordering::Equal => println!("You win!"),
    }
}
