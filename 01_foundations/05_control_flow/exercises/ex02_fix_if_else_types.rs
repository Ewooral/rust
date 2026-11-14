//! Exercise 2: make it compile.
//!
//! `if` is an expression, and a variable can only have one type, so both
//! branches must give the same type (E0308 `if` and `else` have incompatible
//! types). Make both branches give a NUMBER of lives.
//!
//! Run: cargo run -p control_flow --features exercises --bin ex02_fix_if_else_types

fn main() {
    let hard_mode = false;
    let lives = if hard_mode { 1 } else { "three" };
    println!("You start with {lives} lives");
}
