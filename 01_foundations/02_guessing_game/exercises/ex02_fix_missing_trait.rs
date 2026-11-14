//! Exercise 2: make it compile.
//!
//! Roll a die with `rand`. The method exists, but the compiler says it can't
//! find it (E0599). Read the `help:` lines: what has to be "in scope" for a
//! trait's methods to work? Fix it by adding ONE line.
//!
//! Run: cargo run -p guessing_game --features exercises --bin ex02_fix_missing_trait

fn main() {
    let roll = rand::rng().random_range(1..=6);
    println!("You rolled a {roll}");
}
