//! Functions that hand a value back: `->`, the final expression, and `return`.
//! Run: cargo run -p functions_and_comments --example return_values

fn main() {
    let x = five();
    println!("five() = {x}");
    println!("plus_one(5) = {}", plus_one(5));
    println!("safe_divide(7, 2) = {}", safe_divide(7, 2));
    println!("safe_divide(7, 0) = {}", safe_divide(7, 0));

    // A tuple lets one function return several values.
    let (quotient, remainder) = divide_with_remainder(17, 5);
    println!("17 / 5 = {quotient} remainder {remainder}");

    // A function with no `->` returns `()`, the unit value, which means
    // "nothing useful". We call it for what it DOES, not for a value.
    say_hi();
}

// `-> i32` declares the return type. The body's final expression, with no
// semicolon, is the value returned. A function this small is fine!
fn five() -> i32 {
    5
}

fn plus_one(x: i32) -> i32 {
    x + 1
    // Writing `x + 1;` here would be error E0308: the semicolon turns the
    // value into a statement, and the function would return `()`.
}

// `return` leaves the function early. Use it for special cases, and let the
// normal answer be the final expression.
fn safe_divide(a: i32, b: i32) -> i32 {
    if b == 0 {
        return 0; // (`if` is lesson 05)
    }
    a / b
}

fn divide_with_remainder(a: u32, b: u32) -> (u32, u32) {
    (a / b, a % b)
}

fn say_hi() {
    println!("hi!");
}
