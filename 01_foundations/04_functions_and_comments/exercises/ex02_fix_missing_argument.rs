//! Exercise 2: make it compile.
//!
//! `area` needs two arguments, but it is called with one (E0061). The room
//! is 4 metres wide and 3 metres long: fix the call so it prints 12.
//!
//! Run: cargo run -p functions_and_comments --features exercises --bin ex02_fix_missing_argument

fn area(width: u32, length: u32) -> u32 {
    width * length
}

fn main() {
    let room = area(4);
    println!("The room is {room} square metres");
}
