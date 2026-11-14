//! Exercise 5: write three declarative macros.
//!
//! Replace each macro's single placeholder rule with real rules:
//!
//!   - `hashmap!{ k1 => v1, k2 => v2 }` builds a HashMap. Zero pairs and a
//!     trailing comma must both work. (Repetition: `$( … ),*` and `$(,)?`.)
//!   - `max!(a, b, c, …)` gives the largest of one or more values. Make it
//!     RECURSIVE: a rule for one value, and a rule for "first, then the rest"
//!     that calls `max!` on the rest. Each argument must be evaluated ONCE.
//!   - `square_all!(a, b, c)` gives an array of the squares: `[a*a, b*b, c*c]`,
//!     again evaluating each argument once. (Hint: a block per element.)
//!
//! The tests are written; make them pass. Remove the `#![allow]` when done.
//!
//! Test: cargo test -p advanced_features --features exercises --bin ex05_macro_rules

#![allow(unused_macros, unused_mut, unreachable_code, unused_variables)]

use std::collections::HashMap;

macro_rules! hashmap {
    ($($anything:tt)*) => {
        todo!("build a HashMap from `key => value` pairs")
    };
}

macro_rules! max {
    ($($anything:tt)*) => {
        todo!("the largest value, recursively")
    };
}

macro_rules! square_all {
    ($($anything:tt)*) => {
        todo!("an array of squares")
    };
}

fn main() {
    let capitals: HashMap<&str, &str> = hashmap! {
        "Ghana" => "Accra",
        "Kenya" => "Nairobi",
    };
    println!(
        "{} capitals, Ghana's is {}",
        capitals.len(),
        capitals["Ghana"]
    );
    let biggest: i32 = max!(4, 17, 9);
    println!("max!(4, 17, 9) = {biggest}");
    let squares: [i32; 3] = square_all!(1, 2, 3);
    println!("square_all!(1, 2, 3) = {squares:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashmap_with_pairs_and_trailing_comma() {
        let m: HashMap<&str, i32> = hashmap! { "one" => 1, "two" => 2, };
        assert_eq!(m.len(), 2);
        assert_eq!(m["two"], 2);
        let empty: HashMap<i32, i32> = hashmap! {};
        assert!(empty.is_empty());
    }

    #[test]
    fn max_of_one_and_of_many() {
        let only: i32 = max!(7);
        assert_eq!(only, 7);
        let biggest: i32 = max!(3, 9, 4, 1);
        assert_eq!(biggest, 9);
        let word: &str = max!("pear", "apple", "zebra", "kiwi");
        assert_eq!(word, "zebra");
    }

    #[test]
    fn arguments_are_evaluated_once() {
        let mut calls = 0;
        let mut next = || {
            calls += 1;
            calls
        };
        let biggest: i32 = max!(next(), next(), next());
        assert_eq!(biggest, 3);
        let squares: [i32; 2] = square_all!(next(), next());
        assert_eq!(squares, [16, 25]);
        assert_eq!(calls, 5, "each argument should run exactly once");
    }
}
