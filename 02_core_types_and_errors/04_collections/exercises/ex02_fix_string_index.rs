//! Exercise 2: make it compile (E0277).
//!
//! Print each name's first letter. Strings can't be indexed by number, so ask
//! for the first CHARACTER instead. Make sure "Émile" prints 'É', not a broken
//! half of it. (What should happen for the empty name? Don't crash.)
//!
//! Run: cargo run -p collections --features exercises --bin ex02_fix_string_index

fn main() {
    for name in ["Ama", "Émile", "Zoë", ""] {
        let first = name[0];
        println!("{name:?} starts with {first:?}");
    }
}
