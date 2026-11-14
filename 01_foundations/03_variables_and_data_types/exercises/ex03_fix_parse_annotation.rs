//! Exercise 3: make it compile.
//!
//! `parse` can turn text into many different types, and nothing here tells
//! it which one we want (E0284 type annotations needed). Add a type
//! annotation so `guess` is an unsigned 32-bit integer.
//!
//! Run: cargo run -p variables_and_data_types --features exercises --bin ex03_fix_parse_annotation

fn main() {
    let guess = "42".parse().expect("Not a number!");
    println!("You guessed {guess}");
}
