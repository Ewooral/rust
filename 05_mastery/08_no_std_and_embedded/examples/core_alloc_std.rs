//! `core`, `alloc` and `std`: what lives where.
//! Run: cargo run -p no_std_and_embedded --example core_alloc_std
//!
//! This file is `#![no_std]`, like firmware: it starts with only `core`.
//! `extern crate alloc;` adds the heap types. `mod host` pulls in `std`
//! for ONE job, printing on your PC; on a board that would be a UART.

#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cell::Cell;
use core::fmt::Write;
use core::sync::atomic::{AtomicU32, Ordering};
use core::time::Duration;

use no_std_and_embedded::report::TextBuf;

/// The only part of this file that uses std.
mod host {
    extern crate std;

    pub fn say(line: &str) {
        std::println!("{line}");
    }
}

static BOOT_COUNT: AtomicU32 = AtomicU32::new(0);

fn main() {
    // ---- core: needs nothing at all, not even a heap ------------------------------------
    // The language's own types and traits: Option, Result, iterators, slices,
    // str, char, numbers, Cell, atomics, Duration, fmt, PhantomData, Pin...
    let mut readings = [512u16, 498, 530, 505];
    readings.sort_unstable(); // sort_unstable sorts in place: core
    let total: u32 = readings.iter().map(|&r| u32::from(r)).sum();
    let as_text = core::str::from_utf8(b"SM-01").unwrap_or("?");
    let checked = 250u8.checked_add(10); // None, not a panic
    let counter = Cell::new(1);
    counter.set(counter.get() + 1);
    BOOT_COUNT.fetch_add(1, Ordering::Relaxed);
    let timeout = Duration::from_millis(1500); // Duration is core; Instant is std

    // Formatting without a heap: write! into a fixed buffer.
    let mut line: TextBuf<96> = TextBuf::new();
    let _ = write!(
        line,
        "core:  sorted {readings:?}, total {total}, {as_text}, {checked:?}, cell {}, {timeout:?}",
        counter.get()
    );
    host::say(line.as_str());

    // ---- alloc: needs a heap (a global allocator) ------------------------------------------
    let mut log: Vec<String> = Vec::new();
    log.push("boot".to_string());
    log.push(format!(
        "boots so far: {}",
        BOOT_COUNT.load(Ordering::Relaxed)
    ));
    let shared = Rc::new(String::from("shared settings"));
    let also = Rc::clone(&shared);
    let task: Box<dyn Fn(u16) -> u16> = Box::new(|x| x * 2);
    let mut by_name = BTreeMap::new(); // BTreeMap is in alloc...
    by_name.insert("pump", 1);
    by_name.insert("led", 2);
    let mut stable = alloc::vec![(2, 'b'), (1, 'z'), (2, 'a')];
    stable.sort(); // ...and so is the stable sort, because it allocates
    host::say(&format!(
        "alloc: {log:?}, Rc count {}, task(21) = {}, {by_name:?}, {stable:?}",
        Rc::strong_count(&also),
        task(21)
    ));

    // ---- std: needs an operating system ------------------------------------------------------
    // Files, threads, the clock, networking, environment variables, printing,
    // and HashMap (its default hasher asks the OS for random numbers).
    // None of them exist here; each line below is a real error from this file's setup:
    //     println!("hi");                       error: cannot find macro `println` in this scope (no code)
    //     let t = std::time::Instant::now();    E0433: cannot find module or crate `std` in this scope
    //     use alloc::collections::HashMap;      E0432: unresolved import `alloc::collections::HashMap`
    // Without `extern crate alloc;` at the top, `Vec` would fail too:
    //     let v: Vec<u8> = Vec::new();          E0425: cannot find type `Vec` in this scope
    host::say("std:   only inside `mod host` in this file");
}
