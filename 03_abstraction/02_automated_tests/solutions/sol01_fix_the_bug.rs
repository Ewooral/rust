//! Solution to exercise 1: a failing test found a bug. Fix the CODE, not the test.
//!
//! Run the tests, read the failure report (which year? what was expected?),
//! then fix `is_leap_year`. The rule: divisible by 4, except years divisible
//! by 100, except years divisible by 400.
//!
//! Test: cargo test -p automated_tests --features solutions --bin sol01_fix_the_bug

fn is_leap_year(year: u32) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

fn main() {
    for year in [1900, 2000, 2024, 2025] {
        println!("{year}: {}", is_leap_year(year));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_years() {
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(2025));
    }

    #[test]
    fn centuries_are_not_leap_years() {
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2100));
    }

    #[test]
    fn every_fourth_century_is_a_leap_year() {
        assert!(is_leap_year(2000), "2000 was a leap year");
        assert!(is_leap_year(2400));
    }
}
