//! Solution to exercise 2: make it compile (E0106).
//!
//! The result may come from EITHER parameter, so the compiler cannot guess
//! how long it is valid. Add lifetime annotations that say "the result lives
//! as long as both inputs do".
//!
//! Run: cargo run -p generics_traits_lifetimes --features solutions --bin sol02_fix_missing_lifetime

fn longer_name<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.chars().count() >= b.chars().count() {
        a
    } else {
        b
    }
}

fn main() {
    let first = String::from("Ama");
    let second = String::from("Kwabena");
    println!("the longer name is {}", longer_name(&first, &second));
}
