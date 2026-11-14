//! Exercise 4: make it compile.
//!
//! The stadium holds 300 people, but the constant's type can't hold 300.
//! The compiler works out every constant while compiling, notices the
//! overflow, and refuses (E0080). Change the type to the SMALLEST unsigned
//! integer type that fits. (Check the table in the README.)
//!
//! Run: cargo run -p variables_and_data_types --features exercises --bin ex04_fix_const_overflow

const STADIUM_CAPACITY: u8 = 200 + 100;

fn main() {
    println!("The stadium holds {STADIUM_CAPACITY} people");
}
