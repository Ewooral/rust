//! Solution to exercise 6: document it (Beyond the Book).
//!
//! Every public item now has a `///` doc comment, so `missing_docs` is happy.
//!
//! Run: cargo run -p functions_and_comments --features solutions --bin sol06_document_it
//! Test: cargo test -p functions_and_comments --features solutions --bin sol06_document_it

#![deny(missing_docs)]

/// The speed of sound in dry air at 20 °C, in metres per second.
pub const SPEED_OF_SOUND: f64 = 343.0;

/// How far away a storm is, in kilometres, given the number of `seconds`
/// between seeing the lightning and hearing the thunder.
///
/// Light arrives almost instantly, so the delay is the sound's travel time.
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
