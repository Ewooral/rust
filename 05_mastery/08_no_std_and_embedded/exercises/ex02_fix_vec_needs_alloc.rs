//! Exercise 2: make it compile (E0425).
//!
//! This board HAS a heap (on a real chip you'd register one with the
//! `embedded-alloc` crate; on the PC, std's allocator is linked in through
//! `mod host`). But a `#![no_std]` crate still can't see `Vec`, `String` or
//! `format!`: they live in the `alloc` crate, and you must ask for it.
//!
//! Make it compile WITHOUT touching the function bodies:
//!   1. one line at the top of the file to link the `alloc` crate;
//!   2. one `use` line inside `mod firmware` for `Vec`, `String` and
//!      `format` (all three live under `alloc::`).
//!
//! Then read the errors again: why is `format!` an error with no code?
//!
//! Run: cargo run -p no_std_and_embedded --features exercises --bin ex02_fix_vec_needs_alloc

#![no_std]

mod firmware {
    /// Keeps every reading. (A heap lets the log grow; on a chip, ask
    /// yourself how big it could get before you run out of RAM!)
    pub struct Log {
        readings: Vec<u16>,
    }

    impl Log {
        pub fn new() -> Log {
            Log {
                readings: Vec::new(),
            }
        }

        pub fn record(&mut self, reading: u16) {
            self.readings.push(reading);
        }

        pub fn summary(&self) -> String {
            let max = self.readings.iter().max().copied().unwrap_or(0);
            format!("{} readings, highest {max}", self.readings.len())
        }
    }
}

mod host {
    extern crate std;

    pub fn say(line: &str) {
        std::println!("{line}");
    }
}

fn main() {
    let mut log = firmware::Log::new();
    for reading in [431, 425, 470, 402] {
        log.record(reading);
    }
    host::say(&log.summary());
}
