//! Slices borrow a contiguous part of a collection.
//! Run: cargo run -p ownership --example slices

/// Returns the first word of `s`. Taking `&str` means it accepts both
/// `&String` (via deref coercion) and string literals.
fn first_word(s: &str) -> &str {
    s.split(' ').next().unwrap_or("")
}

fn main() {
    let s = String::from("hello world");
    let hello = &s[0..5]; // same as &s[..5]
    let world = &s[6..]; // to the end
    println!("{hello} | {world} | whole: {}", &s[..]);

    let word = first_word(&s);
    // s.clear(); // E0502: cannot clear `s` while `word` borrows from it
    println!("first word: {word}");

    // String literals are already slices: `&'static str`.
    let literal: &str = "literals live in the binary";
    println!("{}", first_word(literal));

    // Array slices work the same way: `&[T]`.
    let a = [1, 2, 3, 4, 5];
    let middle: &[i32] = &a[1..4];
    assert_eq!(middle, &[2, 3, 4]);
    println!("middle = {middle:?}");

    // Careful: string slice indices are BYTE offsets and must fall on
    // UTF-8 character boundaries. `&"héllo"[0..2]` panics at runtime.
    let h = "héllo";
    println!(
        "'{h}' has {} chars but {} bytes",
        h.chars().count(),
        h.len()
    );
}
