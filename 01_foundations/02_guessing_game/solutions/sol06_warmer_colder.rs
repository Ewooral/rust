//! Solution to exercise 6: warmer or colder?
//!
//! Test: cargo test -p guessing_game --features solutions --bin sol06_warmer_colder

use std::cmp::Ordering;

fn warmer_or_colder(previous: u32, current: u32, secret: u32) -> &'static str {
    // `abs_diff` gives the distance without risking a negative u32 (which
    // would overflow and panic in a debug build).
    let before = previous.abs_diff(secret);
    let now = current.abs_diff(secret);
    match now.cmp(&before) {
        Ordering::Less => "Warmer",
        Ordering::Greater => "Colder",
        Ordering::Equal => "Same distance",
    }
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
