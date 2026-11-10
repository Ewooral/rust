//! A reference borrows a value without taking ownership.
//! Run: cargo run -p ownership --example references

// The Book writes `&String` here. Clippy suggests `&str` instead, which also
// accepts string literals; the slices example shows why. Kept as in the Book.
#[allow(clippy::ptr_arg)]
fn calculate_length(s: &String) -> usize {
    s.len()
} // nothing is dropped: we never owned the String

fn change(s: &mut String) {
    s.push_str(", world");
}

fn main() {
    let s1 = String::from("hello");
    let len = calculate_length(&s1);
    println!("'{s1}' has length {len}"); // s1 is still usable

    let mut s = String::from("hello");
    change(&mut s);
    println!("{s}");

    // Many shared borrows at once are fine.
    let r1 = &s;
    let r2 = &s;
    println!("{r1} and {r2}");
    // r1 and r2 are not used after this point, so their borrows end here
    // (non-lexical lifetimes) and a mutable borrow is now allowed.
    let r3 = &mut s;
    r3.push('!');
    println!("{r3}");
}
