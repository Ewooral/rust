//! Solution to exercise 2: make it compile.
//!
//! Every parameter needs an argument, in the same order: width, then length.
//!
//! Run: cargo run -p functions_and_comments --features solutions --bin sol02_fix_missing_argument

fn area(width: u32, length: u32) -> u32 {
    width * length
}

fn main() {
    let room = area(4, 3);
    println!("The room is {room} square metres");
}
