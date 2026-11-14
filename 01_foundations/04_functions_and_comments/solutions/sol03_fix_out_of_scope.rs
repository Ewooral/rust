//! Solution to exercise 3: make it compile.
//!
//! Variables are local to the function (the `{ }`) that declares them.
//! To give `greet` the name, pass it in as an argument.
//!
//! Run: cargo run -p functions_and_comments --features solutions --bin sol03_fix_out_of_scope

fn greet(name: &str) {
    println!("Hello, {name}!");
}

fn main() {
    let name = "Ferris";
    greet(name);
}
