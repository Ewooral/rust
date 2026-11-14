//! The mini project: **Conway's Game of Life for the browser**, with a
//! core that is tested on the PC.
//!
//! | Module | Lesson topic |
//! |---|---|
//! | [`life`] | the pure-Rust core; a flat `#[repr(u8)]` grid JavaScript can read in place |
//! | [`pattern`] | parsing user text into a pattern, with errors instead of panics |
//! | [`host`] | what the core needs from outside (log, random) as a trait; `cfg` picks JS or std |
//! | [`bindings`] | every `#[wasm_bindgen]`: the exported `Universe` class and the JS imports |
//!
//! Build it for the browser with `wasm-pack build --target web` (see the
//! README: not run here, because the `wasm32-unknown-unknown` target isn't
//! installed on this machine), and open `www/index.html`.

pub mod bindings;
pub mod host;
pub mod life;
pub mod pattern;

pub use life::{Cell, Universe};
pub use pattern::{Pattern, PatternError};
