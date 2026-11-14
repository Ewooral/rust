//! Exercise 6: document it (Beyond the Book).
//!
//! `#![deny(missing_docs)]` turns "this public item has no documentation"
//! into a compile error, which many libraries use. Make it compile by adding
//! a `///` doc comment to each public item: say what it is, and its unit.
//! Then run `cargo doc -p functions_and_comments --features exercises
//! --bin ex06_document_it --open` and read your documentation.
//!
//! Run: cargo run -p functions_and_comments --features exercises --bin ex06_document_it
//! Test: cargo test -p functions_and_comments --features exercises --bin ex06_document_it

#![deny(missing_docs)]

pub const SPEED_OF_SOUND: f64 = 343.0;

pub fn thunder_distance_km(seconds: f64) -> f64 {
    seconds * SPEED_OF_SOUND / 1000.0
}

fn main() {
    let seconds = 6.0;
    println!(
        "Thunder {seconds} s after the lightning: the storm is {:.1} km away",
        thunder_distance_km(seconds)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_seconds_is_about_a_kilometre() {
        assert!((thunder_distance_km(3.0) - 1.029).abs() < 1e-9);
    }
}
