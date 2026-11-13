//! Solution to exercise 4 (from the Book's chapter 8 summary): median and mode.
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
//! Test: cargo test -p collections --features solutions --bin sol04_median_and_mode

use std::collections::HashMap;

fn median(numbers: &[i32]) -> Option<f64> {
    if numbers.is_empty() {
        return None;
    }
    let mut sorted = numbers.to_vec(); // a copy: the caller's order is untouched
    sorted.sort();
    let mid = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) {
        // Convert first: adding two large i32s could overflow.
        Some((f64::from(sorted[mid - 1]) + f64::from(sorted[mid])) / 2.0)
    } else {
        Some(f64::from(sorted[mid]))
    }
}

fn mode(numbers: &[i32]) -> Option<i32> {
    let mut counts: HashMap<i32, usize> = HashMap::new();
    for &n in numbers {
        *counts.entry(n).or_insert(0) += 1;
    }
    // Highest count wins; on equal counts the SMALLER value counts as bigger.
    counts
        .into_iter()
        .max_by(|(value_a, count_a), (value_b, count_b)| {
            count_a.cmp(count_b).then(value_b.cmp(value_a))
        })
        .map(|(value, _)| value)
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
