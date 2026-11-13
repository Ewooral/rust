//! Exercise 5: test-driven development (TDD), red -> green -> refactor.
//!
//! The tests are written; the code is not. Work in tiny steps:
//!   RED:      run the tests and watch them fail.
//!   GREEN:    write the SIMPLEST code that passes the first failing test.
//!   REFACTOR: tidy up while every test stays green. Repeat.
//! Run one test at a time with a filter, e.g. `-- one_to_three`.
//!
//! Test: cargo test -p automated_tests --features exercises --bin ex05_tdd_roman_numerals

#![allow(unused_variables)]

/// Converts 1..=3999 to Roman numerals.
fn to_roman(n: u32) -> String {
    todo!()
}

fn main() {
    for n in [1, 4, 9, 14, 40, 90, 400, 1994, 2024] {
        println!("{n} = {}", to_roman(n));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_to_three() {
        assert_eq!(to_roman(1), "I");
        assert_eq!(to_roman(2), "II");
        assert_eq!(to_roman(3), "III");
    }

    #[test]
    fn five_and_ten() {
        assert_eq!(to_roman(5), "V");
        assert_eq!(to_roman(10), "X");
        assert_eq!(to_roman(8), "VIII");
    }

    #[test]
    fn subtractive_forms() {
        assert_eq!(to_roman(4), "IV");
        assert_eq!(to_roman(9), "IX");
        assert_eq!(to_roman(40), "XL");
        assert_eq!(to_roman(90), "XC");
        assert_eq!(to_roman(400), "CD");
        assert_eq!(to_roman(900), "CM");
    }

    #[test]
    fn real_years() {
        assert_eq!(to_roman(1994), "MCMXCIV");
        assert_eq!(to_roman(2024), "MMXXIV");
        assert_eq!(to_roman(3999), "MMMCMXCIX");
    }
}
