//! Solution to exercise 3: make it compile (E0597).
//!
//! `longest` is correct. The problem is in `main`: `result` might point into
//! `string2`, which is dropped at the end of the inner block, before
//! `result` is printed. Fix it by moving ONE line (two possible fixes: find
//! both, and say which one changes what the program means).
//!
//! Run: cargo run -p generics_traits_lifetimes --features solutions --bin sol03_fix_does_not_live_long_enough

fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

fn main() {
    let string1 = String::from("long string is long");
    let result;
    // Fix 1 (used here): move string2 OUT of the inner block, so it lives
    // as long as result. The program's meaning is unchanged.
    let string2 = String::from("xyz");
    {
        result = longest(string1.as_str(), string2.as_str());
    }
    println!("The longest string is {result}");
    // Fix 2: move the println! INTO the inner block. That also compiles, but
    // then `result` is only used inside the block.
}
