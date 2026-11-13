//! The `front_of_house` module (private to the crate: declared with plain
//! `mod` in lib.rs). Its children live in the `src/front_of_house/` folder.
//!
//! This is the modern layout: `front_of_house.rs` next to a
//! `front_of_house/` folder. The older layout is `front_of_house/mod.rs`
//! (see back_of_house).

pub mod hosting;
pub mod serving;
