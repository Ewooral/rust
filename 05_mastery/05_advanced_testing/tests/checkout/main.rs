//! Integration tests for the checkout, as ONE test binary.
//!
//! Every file directly in `tests/` is compiled and linked as its own
//! program. A big project with 40 test files links 40 programs. Putting the
//! tests in modules of one `tests/checkout/main.rs` links one, which is much
//! faster, and lets the modules share `support` without any tricks. Cargo
//! finds `tests/<name>/main.rs` by itself and names the binary `<name>`.
//!
//! Run all of them:   cargo test -p advanced_testing --test checkout
//! Run one module:    cargo test -p advanced_testing --test checkout receipts::

mod flows;
mod fuzz;
mod properties;
mod receipts;
mod support;
