//! Solution to exercise 3: make it compile.
//!
//! An `if` expression with an `else` gives `advice` a value on every path,
//! and the variable is set in one place. (Adding `else { advice = "..."; }`
//! also compiles, but clippy suggests this shape instead.)
//!
//! Run: cargo run -p control_flow --features solutions --bin sol03_fix_uninitialized

fn main() {
    let temperature = 12;
    let advice = if temperature > 20 {
        "wear a T-shirt"
    } else {
        "take a jumper"
    };
    println!("Today: {advice}");
}
