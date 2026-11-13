//! Exercise 1: make it compile (E0369).
//!
//! `smallest` should work for numbers, chars and strings. The compiler does
//! not know that every T can be compared with `<`. Add the trait bound it
//! needs, and nothing else.
//!
//! Run: cargo run -p generics_traits_lifetimes --features exercises --bin ex01_fix_missing_bound

fn smallest<T>(list: &[T]) -> &T {
    let mut smallest = &list[0];
    for item in list {
        if item < smallest {
            smallest = item;
        }
    }
    smallest
}

fn main() {
    println!("{}", smallest(&[34, 50, 25, 100, 65]));
    println!("{}", smallest(&['y', 'm', 'a', 'q']));
    println!("{}", smallest(&["kiwi", "apple", "pear"]));
}
