//! Solution to exercise 2: make it compile (`missing_docs`).
//!
//! Every public item now has a `///` comment: the module, the struct, and
//! both methods. `new` explains when it panics, `fahrenheit` has an example,
//! and the struct's comment links to `new`.
//!
//! Run:  cargo run -p cargo_and_crates_io --features solutions --bin sol02_fix_missing_docs
//! Docs: cargo doc -p cargo_and_crates_io --features solutions --bin sol02_fix_missing_docs --no-deps --open

#![deny(missing_docs)]

/// Measuring the weather.
pub mod weather {
    /// A thermometer reading. Make one with [`Thermometer::new`].
    pub struct Thermometer {
        celsius: f64,
    }

    impl Thermometer {
        /// A reading of `celsius` degrees Celsius.
        ///
        /// # Panics
        ///
        /// Panics if `celsius` is below absolute zero (-273.15 °C), because
        /// no real thermometer can show that.
        pub fn new(celsius: f64) -> Thermometer {
            assert!(celsius >= -273.15, "colder than absolute zero");
            Thermometer { celsius }
        }

        /// The same reading in degrees Fahrenheit.
        ///
        /// # Examples
        ///
        /// ```
        /// let boiling = Thermometer::new(100.0);
        /// assert_eq!(boiling.fahrenheit(), 212.0);
        /// ```
        pub fn fahrenheit(&self) -> f64 {
            self.celsius * 9.0 / 5.0 + 32.0
        }
    }
}

fn main() {
    let today = weather::Thermometer::new(21.5);
    println!("21.5 °C is {:.1} °F", today.fahrenheit());
}
