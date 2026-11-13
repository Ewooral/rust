//! Solution to exercise 2: make it compile (E0277).
//!
//! Print each name's first letter. Strings can't be indexed by number, so ask
//! for the first CHARACTER instead. Make sure "Émile" prints 'É', not a broken
//! half of it. (What should happen for the empty name? Don't crash.)
//!
//! Run: cargo run -p collections --features solutions --bin sol02_fix_string_index

fn main() {
    for name in ["Ama", "Émile", "Zoë", ""] {
        // `chars().next()` is the first character, or None for "".
        let first = name.chars().next();
        println!("{name:?} starts with {first:?}");
    }
}
