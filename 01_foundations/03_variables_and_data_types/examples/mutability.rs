//! Variables are immutable by default; `mut` opts in to change.
//! Run: cargo run -p variables_and_data_types --example mutability

fn main() {
    // Immutable: once `x` has a value, it keeps it for the rest of its life.
    let x = 5;
    println!("x = {x}");
    // Uncomment the next line to see E0384, "cannot assign twice to
    // immutable variable `x`":
    // x = 6;

    // Mutable: `mut` tells the compiler, and every reader, "this will change".
    let mut score = 0;
    println!("score = {score}");
    score += 10;
    score += 5;
    println!("score = {score}");

    // Changing a value through a method needs `mut` too: `push_str` changes
    // the String in place.
    let mut name = String::from("Ferris");
    name.push_str(" the crab");
    println!("name = {name}");

    // `let` can also take apart a pair into two variables at once
    // (destructuring, more in the tuples example).
    let (width, mut height) = (3, 4);
    height *= 2;
    println!("{width} x {height}");
}
