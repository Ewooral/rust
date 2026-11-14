//! `if` is an expression, so it can choose a value.
//! Run: cargo run -p control_flow --example if_in_let

fn main() {
    let condition = true;
    // Each branch is a block whose last expression is its value.
    let number = if condition { 5 } else { 6 };
    println!("The value of number is: {number}");

    // Both branches must produce the SAME type, because `number` can only
    // have one type. Uncomment to see E0308 "`if` and `else` have
    // incompatible types":
    // let number = if condition { 5 } else { "six" };

    // A common use: pick a word.
    let items = 1;
    let noun = if items == 1 { "item" } else { "items" };
    println!("You have {items} {noun} in your basket");

    // It also replaces "declare now, assign later in each branch":
    let temperature = 25;
    let advice = if temperature > 20 {
        "wear a T-shirt"
    } else {
        "take a jumper"
    };
    println!("Today: {advice}");
}
