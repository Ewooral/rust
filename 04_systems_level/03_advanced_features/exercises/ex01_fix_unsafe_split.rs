//! Exercise 1: make it compile (E0133).
//!
//! `split_at_mut` is the Book's safe function built on unsafe code, but
//! someone deleted its `unsafe` block. The compiler now refuses the three
//! operations it can't check for you.
//!
//! 1. Wrap ONLY those operations in one `unsafe { … }` block.
//! 2. Above it, write a `// SAFETY:` comment saying why they can't go wrong
//!    here. (Which line of the function makes that true?)
//!
//! Think about it: if the `assert!` were deleted, the function would still
//! compile. Why would it then be UNSOUND, even though `main` has no `unsafe`?
//!
//! Run: cargo run -p advanced_features --features exercises --bin ex01_fix_unsafe_split

use std::slice;

fn split_at_mut(values: &mut [i32], mid: usize) -> (&mut [i32], &mut [i32]) {
    let len = values.len();
    let ptr = values.as_mut_ptr();

    assert!(mid <= len);

    (
        slice::from_raw_parts_mut(ptr, mid),
        slice::from_raw_parts_mut(ptr.add(mid), len - mid),
    )
}

fn main() {
    let mut scores = [10, 20, 30, 40, 50];
    let (first_half, second_half) = split_at_mut(&mut scores, 2);
    first_half[0] += 1;
    second_half[0] += 1;
    println!("{scores:?}"); // [11, 20, 31, 40, 50]
}
