//! Integer types, integer literals, and what happens when a number gets too big.
//! Run: cargo run -p variables_and_data_types --example integers_and_overflow
//!
//! Then make it overflow on purpose:
//!
//! - debug (panics): cargo run -p variables_and_data_types --example integers_and_overflow -- 255
//! - release (wraps): cargo run -p variables_and_data_types --example integers_and_overflow --release -- 255

fn main() {
    // Signed types (i) hold negatives; unsigned (u) hold 0 and up.
    // The number is how many bits: i8 holds -128 to 127.
    println!("i8:    {} to {}", i8::MIN, i8::MAX);
    println!("u8:    {} to {}", u8::MIN, u8::MAX);
    println!("i32:   {} to {}", i32::MIN, i32::MAX);
    println!("u64:   {} to {}", u64::MIN, u64::MAX);
    println!("i128:  up to {}", i128::MAX);
    println!(
        "usize: up to {} ({} bits on this machine)",
        usize::MAX,
        usize::BITS
    );

    // Literals. `_` is a visual separator only, like a comma in 98,222.
    let decimal = 98_222;
    let hex = 0xff;
    let octal = 0o77;
    let binary = 0b1111_0000;
    let byte = b'A'; // u8 only
    let typed = 57u8; // a type suffix
    println!("{decimal} {hex} {octal} {binary} {byte} {typed}");

    // Integer division drops the remainder, rounding toward zero.
    println!("7 / 2 = {}, -7 / 2 = {}, 7 % 2 = {}", 7 / 2, -7 / 2, 7 % 2);

    // Beyond the Book: say what should happen on overflow, out loud.
    let health: u8 = 250;
    println!("250u8 checked_add(10)     = {:?}", health.checked_add(10));
    println!("250u8 wrapping_add(10)    = {}", health.wrapping_add(10));
    println!("250u8 saturating_add(10)  = {}", health.saturating_add(10));
    println!(
        "250u8 overflowing_add(10) = {:?}",
        health.overflowing_add(10)
    );
    println!("3u8 saturating_sub(5)     = {}", 3u8.saturating_sub(5));

    // Plain `+` on a value only known at run time. Pass 255 to overflow it:
    // a debug build panics, a release build quietly wraps around to 0.
    let arg = std::env::args().nth(1).unwrap_or(String::from("41"));
    let start: u8 = arg.trim().parse().expect("pass a number from 0 to 255");
    let next = start + 1;
    println!("{start} + 1 = {next}");
}
