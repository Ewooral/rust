//! `crate::garden`. Its child module lives in the garden/ folder next to this file.

pub mod vegetables;

pub fn describe() -> String {
    format!("a garden growing {}", vegetables::NAME)
}
