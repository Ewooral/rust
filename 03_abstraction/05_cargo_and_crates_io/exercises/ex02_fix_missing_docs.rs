//! Exercise 2: make it compile (`missing_docs`), by writing documentation.
//!
//! This crate's Cargo.toml sets `missing_docs = "warn"` in its [lints]
//! table. The `#![deny(missing_docs)]` line below turns the warning into an
//! error for this file, as many libraries do before they publish.
//!
//! Write a `///` doc comment for every item the compiler lists. Also:
//! - give `Thermometer::new` a `# Panics` section (when does it panic?),
//! - give `Thermometer::fahrenheit` an `# Examples` section,
//! - link one item to another with an intra-doc link: [`Thermometer::new`].
//!
//! Run:  cargo run -p cargo_and_crates_io --features exercises --bin ex02_fix_missing_docs
//! Then read your docs as a web page:
//!       cargo doc -p cargo_and_crates_io --features exercises --bin ex02_fix_missing_docs --no-deps --open

#![deny(missing_docs)]

pub mod weather {
    pub struct Thermometer {
        celsius: f64,
    }

    impl Thermometer {
        pub fn new(celsius: f64) -> Thermometer {
            assert!(celsius >= -273.15, "colder than absolute zero");
            Thermometer { celsius }
        }

        pub fn fahrenheit(&self) -> f64 {
            self.celsius * 9.0 / 5.0 + 32.0
        }
    }
}

fn main() {
    let today = weather::Thermometer::new(21.5);
    println!("21.5 °C is {:.1} °F", today.fahrenheit());
}
