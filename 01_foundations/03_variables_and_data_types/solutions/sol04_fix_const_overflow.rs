//! Solution to exercise 4: make it compile.
//!
//! A u8 stops at 255. The next size up, u16, holds 0 to 65,535, which is
//! plenty for 300.
//!
//! Run: cargo run -p variables_and_data_types --features solutions --bin sol04_fix_const_overflow

const STADIUM_CAPACITY: u16 = 200 + 100;

fn main() {
    println!("The stadium holds {STADIUM_CAPACITY} people");
}
