//! Solution to exercise 1: make it compile (E0271).
//!
//! A JavaScript chart wants a heat map: one row per hour, one value per
//! sensor. A `Vec<Vec<u8>>` is the obvious Rust shape, but wasm-bindgen
//! can't pass it: JavaScript typed arrays are flat, so there is no
//! "array of Uint8Arrays" conversion. wasm-bindgen checks this even when
//! you compile for the PC.
//!
//! Make `heat_map` return ONE flat `Vec<u8>`, row after row
//! (`hours * sensors` values; value for hour h, sensor s at h * sensors + s).
//! In JavaScript that arrives as a single Uint8Array. Then update `main` to
//! print the rows with `.chunks(sensors as usize)`.
//!
//! Run: cargo run -p webassembly --features solutions --bin sol01_fix_nested_vec

use wasm_bindgen::prelude::*;

/// Value for hour h and sensor s: h * 10 + s.
#[wasm_bindgen]
pub fn heat_map(hours: u8, sensors: u8) -> Vec<u8> {
    // flat_map joins the rows into one Vec; `move` copies h into the inner closure.
    (0..hours)
        .flat_map(|h| (0..sensors).map(move |s| h.wrapping_mul(10).wrapping_add(s)))
        .collect()
}

fn main() {
    let sensors = 4;
    for row in heat_map(3, sensors).chunks(sensors as usize) {
        println!("{row:?}");
    }
}
