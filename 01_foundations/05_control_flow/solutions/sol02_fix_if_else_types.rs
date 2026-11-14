//! Solution to exercise 2: make it compile.
//!
//! Both branches now give an integer, so `lives` has one type.
//!
//! Run: cargo run -p control_flow --features solutions --bin sol02_fix_if_else_types

fn main() {
    let hard_mode = false;
    let lives = if hard_mode { 1 } else { 3 };
    println!("You start with {lives} lives");
}
