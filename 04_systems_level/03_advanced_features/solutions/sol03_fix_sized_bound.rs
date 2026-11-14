//! Solution to exercise 3: make it compile (E0277).
//!
//! `describe` should work for ANY debuggable value behind a reference:
//! a number, a slice, a string slice, even a trait object. The last three
//! are dynamically sized types (`[i32]`, `str`, `dyn Debug`), and every
//! generic parameter is `Sized` unless you say otherwise.
//!
//! Change ONLY the signature of `describe` (one bound, 8 characters).
//! Why is it fine to relax `Sized` here? (Look at how `value` is passed.)
//! Answer: `value` is a REFERENCE, `&T`. A reference to an unsized type is a
//! fat pointer that carries the length (or vtable), so the function never
//! needs to know `T`'s size at compile time.
//!
//! Run: cargo run -p advanced_features --features solutions --bin sol03_fix_sized_bound

use std::fmt::Debug;

fn describe<T: Debug + ?Sized>(value: &T) -> String {
    format!("{value:?} uses {} bytes", size_of_val(value))
}

fn main() {
    println!("{}", describe(&42_u8));
    println!("{}", describe(&[1, 2, 3][..])); // [i32]
    println!("{}", describe("hello")); // str
    let anything: &dyn Debug = &(1.5, 'x');
    println!("{}", describe(anything)); // dyn Debug
}
