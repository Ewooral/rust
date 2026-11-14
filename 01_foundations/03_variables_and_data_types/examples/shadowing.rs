//! Shadowing: a new variable with the same name hides the old one.
//! Run: cargo run -p variables_and_data_types --example shadowing

fn main() {
    let x = 5;
    // A NEW `x`, made from the old one. The old 5 is hidden, not changed.
    let x = x + 1;

    {
        // Shadowing inside an inner scope only lasts until the `}`.
        let x = x * 2;
        println!("inner x = {x}"); // 12
    }

    println!("outer x = {x}"); // 6: the inner shadow has ended

    // Shadowing can change the TYPE, because it is a brand-new variable.
    let spaces = "   "; // &str
    let spaces = spaces.len(); // usize
    println!("spaces = {spaces}");

    // `mut` can't do that: a mutable variable keeps its type for life.
    // Uncomment to see E0308 "expected `&str`, found `usize`":
    // let mut gaps = "   ";
    // gaps = gaps.len();

    // A common real use: read text, then turn it into a number, same name.
    let age = "42";
    let age: u32 = age.parse().expect("age should be a number");
    println!("next year you will be {}", age + 1);
}
