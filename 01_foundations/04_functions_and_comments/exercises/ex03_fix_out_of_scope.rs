//! Exercise 3: make it compile.
//!
//! `greet` tries to use `name`, but `name` belongs to `main`. A function can
//! only see its own variables and its parameters (E0425). Fix it by giving
//! `greet` a parameter, and passing the name in when you call it.
//!
//! Run: cargo run -p functions_and_comments --features exercises --bin ex03_fix_out_of_scope

fn greet() {
    println!("Hello, {name}!");
}

fn main() {
    let name = "Ferris";
    greet();
}
