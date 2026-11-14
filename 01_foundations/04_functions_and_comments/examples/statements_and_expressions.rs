//! Statements do something; expressions produce a value.
//! Run: cargo run -p functions_and_comments --example statements_and_expressions

fn main() {
    // A STATEMENT: `let y = 6;` does something (makes a variable) but has no
    // value of its own, so you can't write `let x = (let y = 6);`.
    let y = 6;

    // EXPRESSIONS evaluate to a value: `5 + 6`, a function call, a macro
    // call, and a whole block in curly brackets.
    let sum = 5 + 6;
    let doubled = double(sum);

    // A block is an expression. Its value is its LAST line, which has no
    // semicolon. `x` only exists inside the block.
    let z = {
        let x = 3;
        x + 1
    };
    println!("y = {y}, sum = {sum}, doubled = {doubled}, z = {z}");

    // Put a semicolon after a block's last line and it becomes a statement:
    // the value is thrown away and the block's value is `()`, the empty
    // "unit" value. In a function that promised an `i32`, that's error E0308
    // (see exercise 1).

    // `if` is an expression too (lesson 05), so it can choose a value.
    let size = if sum > 10 { "big" } else { "small" };
    println!("{sum} is {size}");
}

fn double(n: i32) -> i32 {
    n * 2
}
