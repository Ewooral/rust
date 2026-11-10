//! Exercise 1: fix the move error.
//!
//! This does not compile (E0382). Make it compile and print both lines
//! WITHOUT calling `.clone()`. Hint: does `print_greeting` need to own the name?
//!
//! Run: cargo run -p ownership --features exercises --bin ex01_fix_move

fn print_greeting(name: String) {
    println!("Hello, {name}!");
}

fn main() {
    let name = String::from("Ferris");
    print_greeting(name);
    println!("{name} is the Rust mascot");
}
