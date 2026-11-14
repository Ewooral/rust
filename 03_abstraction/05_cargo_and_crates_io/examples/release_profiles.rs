//! Book 14.1: release profiles. The SAME code, built two ways.
//!
//! Run: cargo run -p cargo_and_crates_io --example release_profiles
//! Then: cargo run -p cargo_and_crates_io --example release_profiles --release
//!
//! `cargo build` uses the `dev` profile: quick to compile, slow to run, with
//! extra safety checks. `cargo build --release` uses the `release` profile:
//! slower to compile, much faster to run. Compare the two outputs.

use std::hint::black_box;
use std::panic;
use std::time::Instant;

fn main() {
    // build.rs forwards these from Cargo (see the build_script_output example).
    // Cargo calls the dev profile's output folder "debug": target/debug/.
    println!("built in:          target/{}/", env!("BUILD_PROFILE"));
    println!("opt-level:         {}", env!("BUILD_OPT_LEVEL"));
    println!("debug info:        {}", env!("BUILD_DEBUG"));
    // debug_assertions is on in dev and off in release. `debug_assert!`
    // checks only run when it is on.
    println!("debug_assertions:  {}", cfg!(debug_assertions));

    // 1. Speed. black_box stops the optimiser from working the answer out
    //    at compile time, so both builds really run the loop.
    let start = Instant::now();
    let mut total: u64 = 0;
    for i in 0..black_box(30_000_000u64) {
        total = total.wrapping_add(i * i % 7);
    }
    println!("loop result:       {total} in {:.0?}", start.elapsed());

    // 2. Overflow checks. In dev, `255u8 + 1` panics. In release the check
    //    is switched off (`overflow-checks = false`) and the number wraps to 0.
    panic::set_hook(Box::new(|_| {})); // hide the panic message for this demo
    let result = panic::catch_unwind(|| black_box(255u8) + black_box(1));
    let _ = panic::take_hook(); // restore the normal panic message
    match result {
        Ok(wrapped) => println!("255u8 + 1:         {wrapped} (wrapped: overflow checks are off)"),
        Err(_) => println!("255u8 + 1:         panicked: attempt to add with overflow"),
    }
    // Don't rely on either: say what you mean with checked_add, wrapping_add
    // or saturating_add, and the answer is the same in every profile.
    println!("checked_add:       {:?}", 255u8.checked_add(1));
}

// Change a profile in the WORKSPACE ROOT Cargo.toml (in a member's
// Cargo.toml it is ignored, with a warning):
//
//     [profile.dev]
//     opt-level = 1      # a little optimisation, still quick to build
//
//     [profile.release]
//     opt-level = 3      # the default
//     lto = true         # optimise across crates: smaller, faster, slower build
//     panic = "abort"    # no unwinding: a smaller binary
