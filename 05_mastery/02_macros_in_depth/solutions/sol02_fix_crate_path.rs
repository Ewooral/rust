//! Solution to exercise 2: make it compile (E0433).
//!
//! `fahrenheit!` lives in the `units` module and calls a helper function
//! there. Paths inside a `macro_rules!` macro are resolved where the macro is
//! USED, so `units::celsius_to_fahrenheit` works from the crate root but not
//! from inside `report`, which has no `units` in scope.
//!
//! Fix the MACRO, not the callers: start the path with `$crate`, which always
//! means "the crate this macro was defined in", from wherever it's used.
//! (In a library with `#[macro_export]`, that's the only way to reach your
//! own helpers from a user's crate.)
//!
//! Run: cargo run -p macros_in_depth --features solutions --bin sol02_fix_crate_path

mod units {
    pub fn celsius_to_fahrenheit(celsius: f64) -> f64 {
        celsius * 9.0 / 5.0 + 32.0
    }

    /// `fahrenheit!(20.0)` converts Celsius to Fahrenheit.
    macro_rules! fahrenheit {
        ($celsius:expr) => {
            $crate::units::celsius_to_fahrenheit($celsius)
        };
    }
    // Make the macro importable by path, like a function.
    pub(crate) use fahrenheit;
}

mod report {
    use crate::units::fahrenheit;

    pub fn line(celsius: f64) -> String {
        format!("{celsius}°C = {}°F", fahrenheit!(celsius))
    }
}

fn main() {
    for celsius in [-40.0, 0.0, 37.0, 100.0] {
        println!("{}", report::line(celsius));
    }
}
