//! Exercise 2: make it compile.
//!
//! We want to reuse the name `spaces` for the NUMBER of spaces. A `mut`
//! variable can change its value but never its type (E0308). Fix it with
//! shadowing, and remove the `mut` that is no longer needed.
//!
//! Run: cargo run -p variables_and_data_types --features exercises --bin ex02_fix_shadow_type_change

fn main() {
    let mut spaces = "   ";
    println!("spaces: {spaces:?}");
    spaces = spaces.len();
    println!("that is {spaces} characters");
}
