//! Compile-fail ("UI") tests: each file in `tests/ui/` must FAIL to compile,
//! with exactly the error messages in the `.stderr` file next to it.
//!
//! Run: cargo test -p macros_in_depth --test compile_fail
//!
//! `trybuild` builds every case as a small crate of its own (under
//! `target/tests/trybuild/`) and compares the compiler's output with the
//! `.stderr` file. After you change a message on purpose, regenerate the
//! files with `TRYBUILD=overwrite` set for one run, then read the diff.
//!
//! These test OUR messages (the `syn::Error`s the macros return), which
//! don't change when Rust is updated. Compiler-written errors such as E0277
//! can change wording between Rust versions, so they're better tested by
//! hand (see the exercises).

#[test]
fn macro_errors_point_at_the_problem() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/*.rs");
}
