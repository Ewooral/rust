//! Exercise 6: warmer or colder?
//!
//! A friendlier hint: compare this guess with the previous one. Return
//! "Warmer" if `current` is closer to `secret` than `previous` was, "Colder"
//! if it is further away, and "Same distance" if it is just as far.
//!
//! Hints: `a.abs_diff(b)` is the distance between two `u32`s, and it never
//! goes negative. Then `cmp` the two distances and `match` the `Ordering`.
//!
//! Test: cargo test -p guessing_game --features exercises --bin ex06_warmer_colder

#![allow(unused_variables)]

fn warmer_or_colder(previous: u32, current: u32, secret: u32) -> &'static str {
    todo!("is {current} closer to {secret} than {previous}?")
}

fn main() {
    let secret = 70;
    let mut previous = 10;
    println!("first guess: {previous}");
    for current in [50, 90, 75, 65] {
        println!("{current}: {}", warmer_or_colder(previous, current, secret));
        previous = current;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closer_is_warmer() {
        assert_eq!(warmer_or_colder(10, 50, 70), "Warmer");
        assert_eq!(warmer_or_colder(100, 71, 70), "Warmer");
    }

    #[test]
    fn further_is_colder() {
        assert_eq!(warmer_or_colder(50, 10, 70), "Colder");
        assert_eq!(warmer_or_colder(69, 100, 70), "Colder");
    }

    #[test]
    fn same_distance_on_either_side() {
        assert_eq!(warmer_or_colder(60, 80, 70), "Same distance");
        assert_eq!(warmer_or_colder(5, 5, 70), "Same distance");
    }
}
