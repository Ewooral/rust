//! Defining and calling functions, with parameters.
//! Run: cargo run -p functions_and_comments --example defining_functions

fn main() {
    println!("Hello from main!");

    // A function can be defined after the code that calls it. Rust only
    // cares that it's defined somewhere the caller can see.
    another_function();

    // Arguments go in the parentheses, in the same order as the parameters.
    print_number(5);
    print_labeled_measurement(5, 'h');

    // An argument can be any expression of the right type.
    print_number(2 + 3 * 4);
}

// `fn`, then the name in snake_case, then `()` and a body in `{}`.
fn another_function() {
    println!("Another function.");
}

// Every parameter needs a name AND a type. The compiler never guesses here.
fn print_number(x: i32) {
    println!("The value of x is: {x}");
}

// Several parameters are separated by commas.
fn print_labeled_measurement(value: i32, unit_label: char) {
    println!("The measurement is: {value}{unit_label}");
}
