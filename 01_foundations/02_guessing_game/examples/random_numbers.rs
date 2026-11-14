//! Random numbers with the `rand` crate (version 0.9).
//! Run: cargo run -p guessing_game --example random_numbers
//!
//! The Book uses rand 0.8 (`rand::thread_rng().gen_range(1..=100)`). In 0.9
//! the same thing is `rand::rng().random_range(1..=100)`, because `gen` is a
//! reserved word in the 2024 edition.

// `Rng` is a TRAIT: a set of methods. The methods only work if the trait is in
// scope, so without this line `.random_range` is "not found" (error E0599).
use rand::Rng;
// `SeedableRng` adds `seed_from_u64`, for repeatable "random" numbers.
use rand::SeedableRng;
use rand::rngs::StdRng;

fn main() {
    // A generator seeded by the operating system: different every run.
    let mut rng = rand::rng();

    // `1..=6` includes 6. `1..7` would mean the same: 7 is excluded.
    let die = rng.random_range(1..=6);
    let percent = rng.random_range(0..100);
    let coin: bool = rng.random();
    println!("die: {die}, percent: {percent}, heads: {coin}");

    // A SEEDED generator: the same seed gives the same numbers on every run
    // (with the same version of rand). Tests and games with "replay" use this.
    let mut first = StdRng::seed_from_u64(42);
    let mut second = StdRng::seed_from_u64(42);
    let a: u32 = first.random_range(1..=100);
    let b: u32 = second.random_range(1..=100);
    println!("seed 42 twice: {a} and {b} (always equal)");
    assert_eq!(a, b);
}
