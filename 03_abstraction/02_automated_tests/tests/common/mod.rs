//! Helpers shared by several integration test files.
//!
//! Why `tests/common/mod.rs` and not `tests/common.rs`? Every `.rs` file
//! directly in `tests/` is compiled as its OWN test crate, so `common.rs`
//! would show up in the output as a test file with 0 tests. Files in a
//! sub-folder are not, so this module is only used where a test file says
//! `mod common;`.

/// Builds a game from the given rolls, padded with gutter balls.
pub fn padded_game(rolls: &[u32]) -> Vec<u32> {
    let mut all = rolls.to_vec();
    all.resize(20, 0);
    all
}

/// Turns "X 7 / 9 - ..." style notation into rolls: X = strike,
/// / = spare (the rest of the pins), - = gutter.
pub fn from_notation(notation: &str) -> Vec<u32> {
    let mut rolls = Vec::new();
    for symbol in notation.split_whitespace() {
        let pins = match symbol {
            "X" => 10,
            "-" => 0,
            "/" => 10 - rolls.last().copied().unwrap_or(0),
            digit => digit.parse().expect("a digit, X, / or -"),
        };
        rolls.push(pins);
    }
    rolls
}
