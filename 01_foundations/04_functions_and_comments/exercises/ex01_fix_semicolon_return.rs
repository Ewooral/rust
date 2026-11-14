//! Exercise 1: make it compile.
//!
//! `plus_one` promises to return an `i32`, but the compiler says it returns
//! `()` (E0308). Read the `help:` line, then fix it by DELETING one
//! character. Why does that one character matter so much?
//!
//! Run: cargo run -p functions_and_comments --features exercises --bin ex01_fix_semicolon_return

fn plus_one(x: i32) -> i32 {
    x + 1;
}

fn main() {
    let x = plus_one(5);
    println!("The value of x is: {x}");
}
