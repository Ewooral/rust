//! The `automated_tests` library for Book chapter 11.
//!
//! Integration tests (the `tests/` folder) can only use a LIBRARY crate, so
//! the real code lives here and `src/main.rs` is a thin program around it.
//!
//! - `bowling`: the mini project, a ten-pin bowling scorer, built test-first.
//! - `doc_tests`: a showcase of everything a documentation test can do.

pub mod bowling;
pub mod doc_tests;
