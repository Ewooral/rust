//! Solution to exercise 4: your first function body.
//!
//! Test: cargo test -p hello_world --features solutions --bin sol04_greeting
//! Run:  cargo run -p hello_world --features solutions --bin sol04_greeting

fn greeting(name: &str) -> String {
    format!("Hello, {name}!")
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
