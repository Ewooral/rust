//! Exercise 3: fix the dangling reference.
//!
//! This does not compile (E0515): the function returns a reference to a
//! value that is dropped when the function ends. Change the function so the
//! caller gets the greeting. Keep the parameter as `&str`.
//!
//! Run: cargo run -p ownership --features exercises --bin ex03_fix_dangling

fn make_greeting(name: &str) -> &str {
    let greeting = format!("Hello, {name}!");
    &greeting
}

fn main() {
    let greeting = make_greeting("Ferris");
    println!("{greeting}");
}
