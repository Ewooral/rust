//! Solution to exercise 3: make it compile.
//!
//! The compiler suggested `print!`, which compiles but would put both
//! messages on one line. We wanted `println!`. Read suggestions, then decide.
//!
//! Run: cargo run -p hello_world --features solutions --bin sol03_fix_macro_name

fn main() {
    println!("Line one");
    println!("Line two");
}
