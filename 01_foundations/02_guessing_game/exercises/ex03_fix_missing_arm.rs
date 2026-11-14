//! Exercise 3: make it compile.
//!
//! A `match` must handle every possible value, and one is missing (E0004).
//! Add the missing arm so a correct guess prints "You win!". Don't use the
//! `_` catch-all: name the case.
//!
//! Run: cargo run -p guessing_game --features exercises --bin ex03_fix_missing_arm

use std::cmp::Ordering;

fn main() {
    let secret_number: u32 = 50;
    let guess: u32 = 50;

    match guess.cmp(&secret_number) {
        Ordering::Less => println!("Too small!"),
        Ordering::Greater => println!("Too big!"),
    }
}
