//! Exercise 3: make it compile.
//!
//! A typo: the compiler cannot find a macro with this name. (This error has
//! no code number.) The compiler suggests a similar name, but is it the one
//! you want? We want each message on its own line.
//!
//! Run: cargo run -p hello_world --features exercises --bin ex03_fix_macro_name

fn main() {
    printn!("Line one");
    printn!("Line two");
}
