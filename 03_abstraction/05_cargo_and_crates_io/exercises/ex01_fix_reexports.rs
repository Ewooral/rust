//! Exercise 1: make it compile (E0432), by fixing the LIBRARY, not its user.
//!
//! `shapes` is a library (written as a module so it fits in one file). Its
//! author tidied the code into `shapes::kinds` and `shapes::measure`. Its
//! users, in `main`, want the short paths `shapes::Circle`, `shapes::Square`
//! and `shapes::area`. Add `pub use` lines INSIDE `mod shapes` so they work.
//! Do not change `main` or the `use` line above it.
//!
//! Run: cargo run -p cargo_and_crates_io --features exercises --bin ex01_fix_reexports

/// A tiny geometry library.
pub mod shapes {
    /// The shapes this library knows.
    pub mod kinds {
        /// A circle.
        pub struct Circle {
            /// Its radius.
            pub radius: f64,
        }

        /// A square.
        pub struct Square {
            /// The length of one side.
            pub side: f64,
        }
    }

    /// Measuring shapes.
    pub mod measure {
        use super::kinds::{Circle, Square};

        /// Anything with an area.
        pub trait Area {
            /// The area.
            fn area(&self) -> f64;
        }

        impl Area for Circle {
            fn area(&self) -> f64 {
                std::f64::consts::PI * self.radius * self.radius
            }
        }

        impl Area for Square {
            fn area(&self) -> f64 {
                self.side * self.side
            }
        }

        /// The area of any shape.
        pub fn area(shape: &impl Area) -> f64 {
            shape.area()
        }
    }
}

use shapes::{Circle, Square, area};

fn main() {
    let circle = Circle { radius: 1.0 };
    let square = Square { side: 2.0 };
    println!("circle: {:.2}", area(&circle));
    println!("square: {:.2}", area(&square));
}
