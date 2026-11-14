//! Mini project for Book 3.1-3.2: a pocket unit converter.
//!
//! Every conversion is one row in a constant array of tuples. Give it an
//! amount and it converts that amount both ways in every row.
//!
//! Run:  cargo run -p variables_and_data_types           (converts 1)
//! Run:  cargo run -p variables_and_data_types -- 26.2   (a marathon)
//! Test: cargo test -p variables_and_data_types

/// Kilometres in one international mile (exact, by definition).
const KM_PER_MILE: f64 = 1.609_344;
/// Kilograms in one pound (exact, by definition).
const KG_PER_POUND: f64 = 0.453_592_37;
/// Centimetres in one inch (exact, by definition).
const CM_PER_INCH: f64 = 2.54;
/// Litres in one US gallon (exact, by definition).
const LITRES_PER_US_GALLON: f64 = 3.785_411_784;

/// One row per conversion: (from unit, to unit, how many `to` make one `from`).
/// An array has a fixed length, and here the compiler checks it is exactly 4.
const CONVERSIONS: [(&str, &str, f64); 4] = [
    ("mi", "km", KM_PER_MILE),
    ("lb", "kg", KG_PER_POUND),
    ("in", "cm", CM_PER_INCH),
    ("US gal", "L", LITRES_PER_US_GALLON),
];

fn main() {
    // Argument 1, or "1" if there isn't one (arguments: Book chapter 12).
    let amount = std::env::args().nth(1).unwrap_or(String::from("1"));
    // Shadowing: the same name, now an f64 instead of a String.
    let amount: f64 = amount
        .trim()
        .parse()
        .expect("please pass a number, like 42 or 2.5");

    println!("Converting {amount}:");
    // `for` visits each row in turn (loops are lesson 05), and the pattern
    // `(from, to, factor)` destructures each tuple into three variables.
    for (from, to, factor) in CONVERSIONS {
        let forward = round2(convert(amount, factor));
        let back = round2(convert_back(amount, factor));
        println!("  {amount} {from} = {forward} {to}    {amount} {to} = {back} {from}");
    }
}

/// From the first unit of a row to the second: miles to km, and so on.
fn convert(amount: f64, factor: f64) -> f64 {
    amount * factor
}

/// The other way round: km to miles, and so on.
fn convert_back(amount: f64, factor: f64) -> f64 {
    amount / factor
}

/// Rounds to 2 decimal places for display, e.g. 1.609344 -> 1.61.
fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Floats are rarely exactly equal, so compare within a tiny margin.
    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn one_mile_is_exactly_1_609344_km() {
        assert!(close(convert(1.0, KM_PER_MILE), 1.609_344));
    }

    #[test]
    fn a_marathon_is_about_42_km() {
        assert_eq!(round2(convert(26.2, KM_PER_MILE)), 42.16);
    }

    #[test]
    fn a_metre_is_about_39_inches() {
        assert_eq!(round2(convert_back(100.0, CM_PER_INCH)), 39.37);
    }

    #[test]
    fn converting_there_and_back_gives_the_same_amount() {
        for (_, _, factor) in CONVERSIONS {
            assert!(close(convert_back(convert(7.5, factor), factor), 7.5));
        }
    }

    #[test]
    fn rounds_to_two_places() {
        assert_eq!(round2(1.234), 1.23);
        assert_eq!(round2(1.006), 1.01);
        assert_eq!(round2(-1.234), -1.23);
        assert_eq!(round2(5.0), 5.0);
    }

    #[test]
    fn every_row_makes_sense() {
        for (from, to, factor) in CONVERSIONS {
            assert_ne!(from, to);
            assert!(factor > 0.0, "{from} -> {to} must have a positive factor");
        }
    }
}
