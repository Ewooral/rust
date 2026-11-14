//! Exercise 1: make it compile (E0425).
//!
//! `check_range!` panics with a helpful message when a value is outside the
//! caller's `min..=max`. But a `macro_rules!` macro looks names up where it
//! was DEFINED (up here, at the top level), not where it's used. So it
//! can't see the `min` and `max` that `main` declares: that's hygiene.
//!
//! Fix the macro so the caller passes the bounds in:
//!     check_range!(speed, min, max);
//! Add two `expr` metavariables to the pattern and use them in the body,
//! then update the two calls. (Defining the macro inside `main`, after the
//! `let`, would also compile, but then it only works there.)
//!
//! Bonus: `$value` is pasted three times, so `speed + 2` is computed three
//! times. Bind it once with `let value = $value;` inside a block.
//!
//! Run: cargo run -p macros_in_depth --features exercises --bin ex01_fix_hygiene

/// Panics unless `min <= value <= max`.
macro_rules! check_range {
    ($value:expr) => {
        if $value < min || $value > max {
            panic!(
                "{} = {} is outside {}..={}",
                stringify!($value),
                $value,
                min,
                max
            );
        }
    };
}

fn main() {
    let (min, max) = (1, 10);
    let speed = 7;
    check_range!(speed);
    check_range!(speed + 2);
    println!("every value is between {min} and {max}");
}
