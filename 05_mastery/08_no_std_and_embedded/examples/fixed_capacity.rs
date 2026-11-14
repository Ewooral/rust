//! Fixed-capacity collections: the `no_std` replacements for Vec and String.
//! Run: cargo run -p no_std_and_embedded --example fixed_capacity
//!
//! The capacity is part of the TYPE (a const generic), so the memory is
//! reserved at compile time and nothing is ever allocated. The price: a
//! push can fail, and you must decide what happens then.
//!
//! (This example uses std to print. The types themselves are no_std.)

use core::fmt::Write;

use no_std_and_embedded::ring::Ring;

fn main() {
    // ---- Our own Ring: full means "push out the oldest" ----------------------------------
    let mut last_three: Ring<u16, 3> = Ring::new();
    for reading in [500, 480, 470, 455, 440] {
        let pushed_out = last_three.push(reading);
        println!(
            "ring: pushed {reading}, pushed out {pushed_out:?}, now {:?}",
            last_three.iter().collect::<Vec<_>>()
        );
    }
    println!("ring: average of the last 3 = {:?}\n", last_three.average());

    // ---- heapless::Vec: full means "no" ---------------------------------------------------
    // push returns Err(item) and hands the item back: nothing is lost silently.
    let mut queue: heapless::Vec<&str, 2> = heapless::Vec::new();
    for job in ["water", "report", "blink"] {
        match queue.push(job) {
            Ok(()) => println!("heapless::Vec: queued {job:?}"),
            Err(rejected) => println!("heapless::Vec: full, got {rejected:?} back"),
        }
    }
    println!("heapless::Vec: {queue:?}, capacity {}\n", queue.capacity());

    // ---- heapless::String: write! into it ----------------------------------------------------
    let mut text: heapless::String<24> = heapless::String::new();
    write!(text, "moisture={}%", 43).unwrap();
    println!(
        "heapless::String: {text:?} ({} of {} bytes)",
        text.len(),
        text.capacity()
    );
    let too_long = write!(text, " and a very long tail");
    println!("heapless::String: writing more -> {too_long:?}\n");

    // ---- heapless::Deque: a double-ended queue ---------------------------------------------------
    let mut deque: heapless::Deque<u8, 4> = heapless::Deque::new();
    deque.push_back(1).unwrap();
    deque.push_back(2).unwrap();
    deque.push_front(0).unwrap();
    let before: Vec<u8> = deque.iter().copied().collect();
    let front = deque.pop_front();
    println!("heapless::Deque: {before:?}, pop_front -> {front:?}");

    // ---- Where the memory is ----------------------------------------------------------------------
    // Everything is inline: the size of the type IS the memory it uses.
    println!(
        "\nsize_of Ring<u16, 3> = {} bytes, heapless::Vec<u16, 8> = {} bytes, std Vec<u16> = {} bytes + heap",
        size_of::<Ring<u16, 3>>(),
        size_of::<heapless::Vec<u16, 8>>(),
        size_of::<Vec<u16>>()
    );
}
