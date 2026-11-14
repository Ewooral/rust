//! Solution to exercise 1: make it compile (E0432), by fixing the LIBRARY.
//!
//! Three `pub use` lines at the top of `mod shapes` re-export the items at
//! the short paths `main` asks for. Nothing moved: `shapes::kinds::Circle`
//! still works too.
//!
//! Run: cargo run -p cargo_and_crates_io --features solutions --bin sol01_fix_reexports

/// A tiny geometry library.
pub mod shapes {
    pub use self::kinds::{Circle, Square};
    pub use self::measure::area;

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
