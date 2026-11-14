//! Exercise 3: make it compile (E0425).
//!
//! In the browser, the time comes from JavaScript's `Date.now()`, imported
//! with wasm-bindgen. That import and `now_ms` are compiled only for
//! WebAssembly (`#[cfg(target_arch = "wasm32")]`), so on the PC `now_ms`
//! doesn't exist, and `main` can't call it.
//!
//! Add a second `now_ms` for every OTHER target, with the same signature:
//!     #[cfg(not(target_arch = "wasm32"))]
//!     fn now_ms() -> f64 { ... }
//! using `std::time::SystemTime::now().duration_since(UNIX_EPOCH)` and
//! `as_secs_f64() * 1000.0`. Exactly one of the two `now_ms` exists in
//! any build, so the rest of the code never needs to know which.
//!
//! (Why not use SystemTime in the browser too? On wasm32-unknown-unknown
//! there is no clock behind it: `SystemTime::now()` panics there.)
//!
//! Run: cargo run -p webassembly --features exercises --bin ex03_fix_cfg_gated_import

#[cfg(target_arch = "wasm32")]
mod js {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = Date)]
        pub fn now() -> f64;
    }
}

/// Milliseconds since 1 January 1970.
#[cfg(target_arch = "wasm32")]
fn now_ms() -> f64 {
    js::now()
}

fn stopwatch(started: f64) -> String {
    format!("{:.1} ms", now_ms() - started)
}

fn main() {
    let started = now_ms();
    let total: u64 = (1..=1_000_000).sum();
    println!("sum = {total}, took {}", stopwatch(started));
    println!("now_ms() says {:.0} ms since 1970", now_ms());
}
