//! Solution to exercise 1: make it compile (E0382).
//!
//! `make_label` hands out its captured String, so it can only be called
//! ONCE (it is FnOnce). We want to call it twice. Change the closure's body
//! so it gives back a NEW String each time, and it becomes Fn.
//! Don't change the two calls.
//!
//! Run: cargo run -p iterators_and_closures --features solutions --bin sol01_fix_fnonce

fn main() {
    let name = String::from("Ferris");
    // Fn: it only READS name, and returns a fresh copy each call.
    let make_label = move || name.clone();
    let first = make_label();
    let second = make_label();
    println!("{first} and {second}");
}
