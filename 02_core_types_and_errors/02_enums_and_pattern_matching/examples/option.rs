//! `Option<T>`: Rust's replacement for null.
//! Run: cargo run -p enums_and_pattern_matching --example option
//!
//! The standard library defines it as:
//!     enum Option<T> { None, Some(T) }
//! It is so common that `Some` and `None` are in scope without `Option::`.

fn find_index(names: &[&str], wanted: &str) -> Option<usize> {
    for (i, name) in names.iter().enumerate() {
        if *name == wanted {
            return Some(i);
        }
    }
    None
}

/// You must say what happens in the None case. Here: treat it as 0.
fn add_or_zero(x: i8, y: Option<i8>) -> i8 {
    x + y.unwrap_or(0)
}

fn main() {
    let some_number = Some(5); // Option<i32>, inferred
    let some_char = Some('e'); // Option<char>
    let absent_number: Option<i32> = None; // None needs a type annotation
    println!("{some_number:?} {some_char:?} {absent_number:?}");

    // An Option<i8> is NOT an i8. You cannot use it as one by accident:
    let x: i8 = 5;
    let y: Option<i8> = Some(5);
    // let sum = x + y; // E0277: cannot add `Option<i8>` to `i8`

    println!("sum = {}", add_or_zero(x, y));
    println!("sum = {}", add_or_zero(x, None));

    let names = ["ama", "kofi", "esi"];
    match find_index(&names, "kofi") {
        Some(i) => println!("kofi is at index {i}"),
        None => println!("kofi is not here"),
    }
    println!("yaw: {:?}", find_index(&names, "yaw"));

    // `.unwrap()` panics on None. Fine in tests and quick experiments,
    // a bug waiting to happen in real code.
    // find_index(&names, "yaw").unwrap(); // panics: called `Option::unwrap()` on a `None` value
}
