//! Exercise 4: encapsulation (Book 18.1).
//!
//! A `Thermometer` records temperatures and keeps the lowest, highest and
//! average reading ready to read at any time, like the Book's
//! `AveragedCollection`. All fields are private, so `record` and `clear` are
//! the only ways in, and they must keep the cached values correct.
//!
//! 1. `record(t)`: store the reading and update `lowest`, `highest`, `average`.
//! 2. `clear()`: forget everything.
//! 3. The getters return `None` while there are no readings.
//!
//! Test: cargo test -p trait_objects_and_oop --features exercises --bin ex04_encapsulated_stats

#![allow(unused_variables, dead_code)]

mod weather {
    pub struct Thermometer {
        readings: Vec<f64>,
        lowest: f64,
        highest: f64,
        average: f64,
    }

    impl Thermometer {
        pub fn new() -> Self {
            Thermometer {
                readings: Vec::new(),
                lowest: 0.0,
                highest: 0.0,
                average: 0.0,
            }
        }

        pub fn record(&mut self, celsius: f64) {
            todo!()
        }

        pub fn clear(&mut self) {
            todo!()
        }

        pub fn count(&self) -> usize {
            self.readings.len()
        }

        pub fn lowest(&self) -> Option<f64> {
            todo!()
        }

        pub fn highest(&self) -> Option<f64> {
            todo!()
        }

        pub fn average(&self) -> Option<f64> {
            todo!()
        }
    }
}

use weather::Thermometer;

fn main() {
    let mut garden = Thermometer::new();
    for t in [12.0, 18.0, 9.0] {
        garden.record(t);
    }
    println!(
        "{} readings: lowest {:?}, highest {:?}, average {:?}",
        garden.count(),
        garden.lowest(),
        garden.highest(),
        garden.average()
    );
    // garden.average = 100.0; // E0616: the field is private. Good!
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_empty() {
        let t = Thermometer::new();
        assert_eq!(t.count(), 0);
        assert_eq!(t.lowest(), None);
        assert_eq!(t.highest(), None);
        assert_eq!(t.average(), None);
    }

    #[test]
    fn keeps_the_stats_up_to_date() {
        let mut t = Thermometer::new();
        for reading in [12.0, 18.0, 9.0] {
            t.record(reading);
        }
        assert_eq!(t.lowest(), Some(9.0));
        assert_eq!(t.highest(), Some(18.0));
        assert_eq!(t.average(), Some(13.0));

        t.record(-5.0);
        assert_eq!(t.count(), 4);
        assert_eq!(t.lowest(), Some(-5.0));
        assert_eq!(t.average(), Some(8.5));
    }

    #[test]
    fn clear_forgets_everything() {
        let mut t = Thermometer::new();
        t.record(30.0);
        t.clear();
        assert_eq!(t.count(), 0);
        assert_eq!(t.highest(), None);

        t.record(-2.0); // the old 30 must not linger as the highest
        assert_eq!(t.highest(), Some(-2.0));
        assert_eq!(t.lowest(), Some(-2.0));
    }
}
