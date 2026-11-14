//! Solution to exercise 2: make it compile (E0277).
//!
//! A web page calls `click(button)` from JavaScript, and Rust keeps a count
//! per button between calls, in a global. A `static` must be `Sync` (safe to
//! share between threads), and a `RefCell` isn't: its borrow check isn't
//! thread-safe. "But the browser runs my wasm on one thread!" Maybe, but the
//! same code also compiles for the PC, and WebAssembly can have threads too,
//! so Rust's rules don't bend.
//!
//! Fix it ONE of these ways:
//!   a) `thread_local!`: one `RefCell` per thread, so no sharing at all:
//!        thread_local! { static CLICKS: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) }; }
//!      and inside `click`: `CLICKS.with_borrow_mut(|clicks| { ... })`.
//!   b) `std::sync::Mutex<Vec<u32>>`, which IS Sync (`Mutex::new` is a const fn).
//!
//! Better still, in a real app: no global at all. Keep the counts in an
//! exported `#[wasm_bindgen]` struct, and let JavaScript hold the instance.
//!
//! Run: cargo run -p webassembly --features solutions --bin sol02_fix_global_state

use std::cell::RefCell;

use wasm_bindgen::prelude::*;

// Fix (a): one RefCell per thread. In the browser there is one thread, so
// one counter list, and nothing is ever shared.
thread_local! {
    static CLICKS: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) };
}

/// Counts a click on `button` and returns its new total.
#[wasm_bindgen]
pub fn click(button: usize) -> u32 {
    CLICKS.with_borrow_mut(|clicks| {
        if clicks.len() <= button {
            clicks.resize(button + 1, 0);
        }
        clicks[button] += 1;
        clicks[button]
    })
}

fn main() {
    for button in [0, 2, 0, 0, 2] {
        println!("button {button}: {} clicks", click(button));
    }
}
