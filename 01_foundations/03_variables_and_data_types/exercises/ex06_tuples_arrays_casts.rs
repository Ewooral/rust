//! Exercise 6: tuples, arrays and conversions.
//!
//! Implement the four functions. No loops needed: index the arrays directly.
//!
//! - `swap`: return the pair the other way round.
//! - `weekend`: return the last two days of the week as a tuple.
//! - `average`: the mean of four whole-number scores, as an `f64`. Careful:
//!   `7 / 2` is 3 for integers, and adding four huge u32s overflows. Convert
//!   each score to `f64` first (`f64::from(x)` or `x as f64`), then add.
//! - `to_byte`: `Some(n as a u8)` only if it fits in a u8, otherwise `None`.
//!   Don't use `as` here (why not?). Hint: `u8::try_from(n)` returns a
//!   `Result`, and `.ok()` turns a `Result` into an `Option`.
//!
//! Test: cargo test -p variables_and_data_types --features exercises --bin ex06_tuples_arrays_casts

#![allow(unused_variables)]

fn swap(pair: (i32, char)) -> (char, i32) {
    todo!("swap {pair:?}")
}

fn weekend(days: [&str; 7]) -> (&str, &str) {
    todo!("the last two of {days:?}")
}

fn average(scores: [u32; 4]) -> f64 {
    todo!("the mean of {scores:?}")
}

fn to_byte(n: i32) -> Option<u8> {
    todo!("{n} as a u8, if it fits")
}

fn main() {
    let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    println!("{:?}", swap((1, 'a')));
    println!("{:?}", weekend(days));
    println!("{}", average([7, 8, 8, 8]));
    println!("{:?} {:?}", to_byte(200), to_byte(300));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swaps_a_pair() {
        assert_eq!(swap((1, 'a')), ('a', 1));
        assert_eq!(swap((-7, 'z')), ('z', -7));
    }

    #[test]
    fn finds_the_weekend() {
        let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
        assert_eq!(weekend(days), ("Sat", "Sun"));
    }

    #[test]
    fn average_keeps_the_fraction() {
        assert_eq!(average([7, 8, 8, 8]), 7.75);
        assert_eq!(average([1, 2, 3, 4]), 2.5);
        assert_eq!(average([0, 0, 0, 0]), 0.0);
    }

    #[test]
    fn average_of_big_scores_does_not_overflow() {
        let big = u32::MAX;
        assert_eq!(average([big, big, big, big]), big as f64);
    }

    #[test]
    fn to_byte_only_when_it_fits() {
        assert_eq!(to_byte(0), Some(0));
        assert_eq!(to_byte(255), Some(255));
        assert_eq!(to_byte(256), None);
        assert_eq!(to_byte(-1), None);
    }
}
