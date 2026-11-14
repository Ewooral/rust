//! Solution to exercise 2: make it compile.
//!
//! `let` makes a brand-new variable, so it may have a new type. The first
//! `spaces` (a `&str`) is hidden by the second (a `usize`).
//!
//! Run: cargo run -p variables_and_data_types --features solutions --bin sol02_fix_shadow_type_change

fn main() {
    let spaces = "   ";
    println!("spaces: {spaces:?}");
    let spaces = spaces.len();
    println!("that is {spaces} characters");
}
