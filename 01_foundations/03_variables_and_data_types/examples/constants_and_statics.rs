//! Constants (`const`) and, beyond the Book, `static`.
//! Run: cargo run -p variables_and_data_types --example constants_and_statics

use std::sync::atomic::{AtomicU32, Ordering};

// A CONSTANT: always immutable, type always written, name in
// SCREAMING_SNAKE_CASE. The compiler works the value out while compiling,
// so the arithmetic below costs nothing when the program runs.
const THREE_HOURS_IN_SECONDS: u32 = 60 * 60 * 3;
const BOARD_SIZE: usize = 3;

// A STATIC: exactly one value at one fixed place in memory, for the whole
// run of the program. Read-only statics are like constants with an address.
static APP_NAME: &str = "Data Types Demo";

// A static that changes must be safe to share between threads. An atomic
// integer is (atomics are lesson 04/01). `static mut` exists, but every use
// of it needs `unsafe` (error E0133 without it), so avoid it.
static VISITS: AtomicU32 = AtomicU32::new(0);

fn record_visit() -> u32 {
    VISITS.fetch_add(1, Ordering::Relaxed) + 1
}

fn main() {
    println!("{APP_NAME}");
    println!("three hours = {THREE_HOURS_IN_SECONDS} seconds");

    // Constants can go where the compiler needs a value at compile time,
    // such as the length of an array. A `let` variable could not.
    let board = [['.'; BOARD_SIZE]; BOARD_SIZE];
    println!("board: {board:?}");

    // Constants can be declared in any scope, including inside a function.
    const MAX_POINTS: u32 = 100_000;
    println!("max points: {MAX_POINTS}");

    record_visit();
    record_visit();
    println!("visits so far: {}", record_visit());
}
