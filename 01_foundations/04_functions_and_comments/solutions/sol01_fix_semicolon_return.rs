//! Solution to exercise 1: make it compile.
//!
//! `x + 1` is an expression with a value. `x + 1;` is a statement: the `;`
//! throws the value away, so the body ended with nothing and returned `()`.
//! Without the semicolon, `x + 1` is the function's return value.
//!
//! Run: cargo run -p functions_and_comments --features solutions --bin sol01_fix_semicolon_return

fn plus_one(x: i32) -> i32 {
    x + 1
}

fn main() {
    let x = plus_one(5);
    println!("The value of x is: {x}");
}
