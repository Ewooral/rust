//! Exercise 3: make it compile.
//!
//! `advice` only gets a value when the `if` condition is true. On a cold day
//! it would be used without ever being set, and Rust refuses (E0381).
//! Fix it so a cold day gives "take a jumper". Best of all: turn the whole
//! thing into ONE `let advice = if ... { ... } else { ... };`.
//!
//! Run: cargo run -p control_flow --features exercises --bin ex03_fix_uninitialized

fn main() {
    let temperature = 12;
    let advice;
    if temperature > 20 {
        advice = "wear a T-shirt";
    }
    println!("Today: {advice}");
}
