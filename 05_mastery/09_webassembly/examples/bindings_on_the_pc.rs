//! The `#[wasm_bindgen]` API, called from Rust on the PC.
//! Run: cargo run -p webassembly --example bindings_on_the_pc
//!
//! Calling Rust from JavaScript: `#[wasm_bindgen]` on a function, struct or
//! impl EXPORTS it. In JavaScript the code below would read:
//!
//!     const u = Universe.fromPattern("OOO", 5, 3);
//!     u.tick();
//!     console.log(u.render(), u.population, patternNames());
//!     u.free();
//!
//! Calling JavaScript from Rust: `#[wasm_bindgen] extern "C" { ... }`
//! IMPORTS a JavaScript function, like `console.log`.
//!
//! On the PC the exports are ordinary Rust. The imports have no JavaScript
//! to call, so they panic: that's why the core talks to a `Host` trait
//! instead, and `cfg(target_arch = "wasm32")` picks the real one.

use webassembly::bindings::{self, WasmUniverse};
use webassembly::host::{DefaultHost, Host, ScriptedHost};

fn main() {
    // ---- Exports: plain Rust here --------------------------------------------------------
    let mut universe = WasmUniverse::from_pattern("!Name: Blinker\nOOO", 5, 3).unwrap();
    println!("fromPattern(\"OOO\", 5, 3):\n{}", universe.render());
    let changed = universe.tick();
    println!("tick() changed {changed} cells:\n{}", universe.render());
    println!(
        "getters: width={} height={} generation={} population={}",
        universe.width(),
        universe.height(),
        universe.generation(),
        universe.population()
    );
    println!("patternNames() = {:?}", bindings::pattern_names());

    // cellsPtr: JavaScript would wrap this address in a Uint8Array.
    let cells = (universe.width() * universe.height()) as usize;
    // SAFETY: the universe owns width * height initialised one-byte cells there.
    let bytes = unsafe { std::slice::from_raw_parts(universe.cells_ptr(), cells) };
    println!("the bytes JavaScript would see at cellsPtr(): {bytes:?}\n");

    // ---- Imports: no JavaScript here, so they panic ----------------------------------------
    let quiet = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {})); // hide the panic message
    let result = std::panic::catch_unwind(|| bindings::console_log("hello"));
    std::panic::set_hook(quiet);
    println!(
        "bindings::console_log on the PC panicked: {}",
        result.is_err()
    );

    // ---- So the core uses a Host trait ---------------------------------------------------------
    let mut host = DefaultHost::default(); // StdHost here, JsHost in the browser
    let roll = host.random();
    host.log(&format!("DefaultHost on this target, random() = {roll:.3}"));
    let mut scripted = ScriptedHost::new(&[0.5]);
    scripted.log("tests use a ScriptedHost");
    println!("ScriptedHost kept its logs: {:?}", scripted.logs);
    println!(
        "this build: target_arch = {:?}",
        if cfg!(target_arch = "wasm32") {
            "wasm32"
        } else {
            std::env::consts::ARCH
        }
    );
}
