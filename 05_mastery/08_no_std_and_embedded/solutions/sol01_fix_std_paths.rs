//! Solution to exercise 1: make it compile (E0433).
//!
//! This file is `#![no_std]`, like firmware. The `firmware` module was
//! copied from a desktop program, and it still says `std::` everywhere.
//! There is no `std` here (only `mod host` borrows it, to print on the PC).
//!
//! Fix the `firmware` module only. Every item it uses also exists in `core`,
//! under the same path: `std` re-exports them from there. So the fix is the
//! same small change in four places. Don't add `extern crate std`!
//!
//! Then think: which of these could NOT be fixed this way, and why?
//! `std::time::Instant`, `std::collections::HashMap`, `std::fs::read`.
//!
//! Answer: all three need an operating system. `Instant` reads the OS
//! clock (a board uses its own timer, e.g. embassy_time::Instant), the
//! default HashMap hasher asks the OS for random seeds (use BTreeMap from
//! alloc, or heapless::FnvIndexMap), and there are no files without an OS.
//!
//! Run: cargo run -p no_std_and_embedded --features solutions --bin sol01_fix_std_paths

#![no_std]

mod firmware {
    use core::cmp::Ordering;
    use core::fmt::Write;
    use core::time::Duration;

    /// How long the sensor needs after power-on.
    pub const SETTLE_TIME: Duration = Duration::from_millis(250);

    pub fn trend(previous: u16, now: u16) -> &'static str {
        match now.cmp(&previous) {
            Ordering::Less => "falling",
            Ordering::Equal => "steady",
            Ordering::Greater => "rising",
        }
    }

    pub fn describe<W: Write>(out: &mut W, previous: u16, now: u16) -> core::fmt::Result {
        write!(out, "{now} per mille, {}", trend(previous, now))
    }
}

mod host {
    extern crate std;

    pub fn say(line: &str) {
        std::println!("{line}");
    }
}

fn main() {
    let mut line: no_std_and_embedded::report::TextBuf<64> = Default::default();
    firmware::describe(&mut line, 431, 425).unwrap();
    host::say(line.as_str());
    let mut settle: no_std_and_embedded::report::TextBuf<32> = Default::default();
    core::fmt::Write::write_fmt(
        &mut settle,
        format_args!("settle for {:?}", firmware::SETTLE_TIME),
    )
    .unwrap();
    host::say(settle.as_str());
}
