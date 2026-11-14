//! Exercise 4: your first function body.
//!
//! Implement `greeting` so that `greeting("Ama")` returns `"Hello, Ama!"`.
//! You will learn functions properly in lesson 04; for now, fill in the body.
//! Hint: `format!` works exactly like `println!`, but returns the text as a
//! `String` instead of printing it.
//!
//! Test: cargo test -p hello_world --features exercises --bin ex04_greeting
//! Run:  cargo run -p hello_world --features exercises --bin ex04_greeting

#![allow(unused_variables)]

fn greeting(name: &str) -> String {
    todo!("return \"Hello, <name>!\" for {name}")
}

fn main() {
    println!("{}", greeting("world"));
    println!("{}", greeting("Ferris"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greets_the_world() {
        assert_eq!(greeting("world"), "Hello, world!");
    }

    #[test]
    fn greets_a_name() {
        assert_eq!(greeting("Ama"), "Hello, Ama!");
    }

    #[test]
    fn keeps_the_name_exactly() {
        assert_eq!(greeting("Dr. Ferris Crab"), "Hello, Dr. Ferris Crab!");
    }
}
