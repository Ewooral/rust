//! Solution to exercise 5: test-driven development (TDD), red -> green -> refactor.
//!
//! The tests are written; the code is not. Work in tiny steps:
//!   RED:      run the tests and watch them fail.
//!   GREEN:    write the SIMPLEST code that passes the first failing test.
//!   REFACTOR: tidy up while every test stays green. Repeat.
//! Run one test at a time with a filter, e.g. `-- one_to_three`.
//!
//! Test: cargo test -p automated_tests --features solutions --bin sol05_tdd_roman_numerals

/// Converts 1..=3999 to Roman numerals.
fn to_roman(n: u32) -> String {
    // The refactored end result: a table, largest value first, including
    // the subtractive pairs, so the loop never needs special cases.
    const TABLE: [(u32, &str); 13] = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut n = n;
    let mut out = String::new();
    for (value, symbol) in TABLE {
        while n >= value {
            out.push_str(symbol);
            n -= value;
        }
    }
    out
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
