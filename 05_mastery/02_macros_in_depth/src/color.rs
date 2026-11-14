//! `Rgb`, the colour type the `palette!` macro makes constants of.

use std::fmt;

/// A 24-bit colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    /// Red, 0-255.
    pub r: u8,
    /// Green, 0-255.
    pub g: u8,
    /// Blue, 0-255.
    pub b: u8,
}

impl Rgb {
    /// A colour from its three channels. `const`, so `palette!` can make
    /// constants with it.
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Rgb { r, g, b }
    }

    /// `text` wrapped in the terminal escape codes for this colour.
    pub fn paint(self, text: &str) -> String {
        format!("\x1b[38;2;{};{};{}m{text}\x1b[0m", self.r, self.g, self.b)
    }
}

/// `#rrggbb`.
impl fmt::Display for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}
