//! Exercise 4 (from the Book's chapter 8 summary): median and mode.
//!
//! - median: the middle value once sorted. For an even count, the average of
//!   the two middle values. `None` for an empty list.
//! - mode: the value that appears most often. On a tie, the SMALLEST of the
//!   tied values. `None` for an empty list.
//!
//! You will need: a sorted copy (`to_vec()` then `sort()`), and a HashMap
//! counting each value (`entry(..).or_insert(0)`).
//! Do not change the order of the caller's slice.
//!
//! Test: cargo test -p collections --features exercises --bin ex04_median_and_mode

#![allow(unused_variables)]

use std::collections::HashMap;

fn median(numbers: &[i32]) -> Option<f64> {
    todo!()
}

fn mode(numbers: &[i32]) -> Option<i32> {
    todo!()
}

fn main() {
    let data = [3, 7, 1, 7, 2, 9, 3, 7];
    println!("data {data:?}");
    println!("median {:?}, mode {:?}", median(&data), mode(&data));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn median_odd_and_even() {
        assert_eq!(median(&[5, 1, 3]), Some(3.0));
        assert_eq!(median(&[4, 1, 3, 2]), Some(2.5));
        assert_eq!(median(&[7]), Some(7.0));
        assert_eq!(median(&[]), None);
    }

    #[test]
    fn median_does_not_reorder_input() {
        let data = [3, 1, 2];
        median(&data);
        assert_eq!(data, [3, 1, 2]);
    }

    #[test]
    fn mode_most_common_smallest_on_tie() {
        assert_eq!(mode(&[1, 2, 2, 3]), Some(2));
        assert_eq!(mode(&[4, 4, 1, 1, 9]), Some(1));
        assert_eq!(mode(&[-3]), Some(-3));
        assert_eq!(mode(&[]), None);
    }
}
