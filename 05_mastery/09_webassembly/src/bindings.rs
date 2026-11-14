//! The JavaScript-facing layer: every `#[wasm_bindgen]` in the crate.
//!
//! It is deliberately thin. Each function converts between JavaScript-
//! friendly types (numbers, strings, a class) and the core's types, and the
//! core does the work.
//!
//! What `wasm-pack build --target web` would generate from this file (not
//! run here, see the README) is a JavaScript module like:
//!
//! ```js
//! export default function init() { ... }        // loads and starts the .wasm
//! export class Universe {
//!     constructor(width, height) { ... }
//!     static fromPattern(text, width, height) { ... }   // throws on a bad pattern
//!     tick() { ... }  get width() { ... }  cellsPtr() { ... }  free() { ... }
//! }
//! export function patternNames() { ... }       // an array of strings
//! ```
//!
//! On the PC all of this is plain Rust: the exports can be called and
//! tested; only the IMPORTS (console.log, Math.random) panic if called.

use wasm_bindgen::prelude::*;

use crate::host::{DefaultHost, Host};
use crate::life::Universe;
use crate::pattern;

// ---- Calling JavaScript from Rust: imports ------------------------------------------------
#[wasm_bindgen]
extern "C" {
    /// JavaScript's `console.log`.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn console_log(message: &str);

    /// JavaScript's `Math.random()`.
    #[wasm_bindgen(js_namespace = Math, js_name = random)]
    pub fn math_random() -> f64;
}

// ---- Calling Rust from JavaScript: exports ------------------------------------------------
/// The universe, as a JavaScript class called `Universe`.
///
/// JavaScript holds only a pointer to it; the struct itself lives in the
/// WebAssembly module's memory. When JavaScript is done it must call
/// `universe.free()`, or the memory is never given back.
#[wasm_bindgen(js_name = Universe)]
pub struct WasmUniverse {
    inner: Universe,
    host: DefaultHost,
}

#[wasm_bindgen(js_class = Universe)]
impl WasmUniverse {
    /// `new Universe(width, height)`: an empty grid.
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> WasmUniverse {
        WasmUniverse {
            inner: Universe::new(width, height),
            host: DefaultHost::default(),
        }
    }

    /// `Universe.fromPattern(text, width, height)`: a grid with the pattern
    /// in the middle. A bad pattern becomes a JavaScript `Error` (thrown),
    /// thanks to `Result<_, JsError>`: `?` converts any std error.
    #[wasm_bindgen(js_name = fromPattern)]
    pub fn from_pattern(text: &str, width: u32, height: u32) -> Result<WasmUniverse, JsError> {
        let pattern = pattern::parse(text)?;
        let mut universe = WasmUniverse::new(width.max(pattern.width), height.max(pattern.height));
        let row = (universe.inner.height() - pattern.height) / 2;
        let col = (universe.inner.width() - pattern.width) / 2;
        universe.inner.insert(&pattern, row, col);
        Ok(universe)
    }

    /// One generation; returns how many cells changed.
    pub fn tick(&mut self) -> u32 {
        self.inner.tick()
    }

    /// Several generations in ONE call. Crossing the JavaScript/WebAssembly
    /// boundary is cheap, but not free: batch work where you can.
    #[wasm_bindgen(js_name = tickMany)]
    pub fn tick_many(&mut self, generations: u32) -> u32 {
        (0..generations).map(|_| self.inner.tick()).sum()
    }

    /// Flips a cell (a mouse click).
    pub fn toggle(&mut self, row: u32, col: u32) {
        self.inner.toggle(row, col);
    }

    /// Random cells, using `Math.random` in the browser.
    pub fn randomize(&mut self, density: f64) {
        self.inner.randomize(&mut self.host, density);
        self.host.log(&format!(
            "randomized: {} cells alive",
            self.inner.population()
        ));
    }

    /// Kills every cell.
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Stamps a built-in pattern; `false` if there is no such pattern.
    #[wasm_bindgen(js_name = insertPattern)]
    pub fn insert_pattern(&mut self, name: &str, row: u32, col: u32) -> bool {
        match pattern::builtin(name) {
            Some(pattern) => {
                self.inner.insert(&pattern, row, col);
                true
            }
            None => false,
        }
    }

    /// `universe.width` (a getter, so no brackets in JavaScript).
    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.inner.width()
    }

    /// `universe.height`.
    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.inner.height()
    }

    /// `universe.generation`. A `u64` would arrive in JavaScript as a
    /// `BigInt`, which doesn't mix with ordinary numbers; an `f64` is exact
    /// up to 2^53, about 285,000 years at 1000 generations a second.
    #[wasm_bindgen(getter)]
    pub fn generation(&self) -> f64 {
        self.inner.generation() as f64
    }

    /// `universe.population`.
    #[wasm_bindgen(getter)]
    pub fn population(&self) -> u32 {
        // At most 512 x 512 cells in practice; saturate rather than wrap.
        u32::try_from(self.inner.population()).unwrap_or(u32::MAX)
    }

    /// Where the cells start in WebAssembly memory: one byte per cell, row
    /// after row. JavaScript reads them WITHOUT copying:
    /// `new Uint8Array(memory.buffer, universe.cellsPtr(), width * height)`.
    #[wasm_bindgen(js_name = cellsPtr)]
    pub fn cells_ptr(&self) -> *const u8 {
        self.inner.cells_ptr().cast()
    }

    /// The grid as text: `#` alive, `.` dead.
    pub fn render(&self) -> String {
        self.inner.to_string()
    }
}

/// `patternNames()`: the built-in patterns, as a JavaScript array of strings.
#[wasm_bindgen(js_name = patternNames)]
pub fn pattern_names() -> Vec<String> {
    pattern::builtin_names().map(String::from).collect()
}

/// Runs once, as soon as the module is loaded (`#[wasm_bindgen(start)]`).
#[wasm_bindgen(start)]
pub fn start() {
    DefaultHost::default().log("webassembly lesson: Rust module loaded");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_exported_class_works_as_plain_rust() {
        let mut universe = WasmUniverse::new(8, 8);
        assert!(universe.insert_pattern("blinker", 3, 2));
        assert!(!universe.insert_pattern("dragon", 0, 0));
        assert_eq!(universe.population(), 3);
        assert_eq!(universe.tick_many(2), 8, "4 changes each tick");
        assert_eq!(universe.generation(), 2.0);
        assert_eq!((universe.width(), universe.height()), (8, 8));
    }

    #[test]
    fn from_pattern_centres_the_pattern() {
        let universe = WasmUniverse::from_pattern("OOO", 5, 3).unwrap();
        assert_eq!(universe.render(), ".....\n.###.\n.....\n");
        let tiny = WasmUniverse::from_pattern("OOOO", 1, 1).unwrap();
        assert_eq!(tiny.width(), 4, "grows to fit the pattern");
    }

    #[test]
    fn cells_ptr_points_at_one_byte_per_cell() {
        let universe = WasmUniverse::from_pattern("O.\n.O", 2, 2).unwrap();
        // SAFETY: a 2x2 universe owns 4 initialised bytes at this address.
        let bytes = unsafe { std::slice::from_raw_parts(universe.cells_ptr(), 4) };
        assert_eq!(bytes, [1, 0, 0, 1]);
    }

    #[test]
    fn pattern_names_lists_the_builtins() {
        assert_eq!(
            pattern_names(),
            ["glider", "blinker", "toad", "lwss", "pulsar"]
        );
    }

    #[test]
    fn randomize_on_the_pc_uses_the_std_host() {
        let mut universe = WasmUniverse::new(20, 20);
        universe.randomize(0.5);
        assert!((100..300).contains(&universe.population()));
    }
}
