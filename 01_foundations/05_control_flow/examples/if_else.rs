//! `if`, `else if` and `else`: run code only when a condition is true.
//! Run: cargo run -p control_flow --example if_else

fn main() {
    let number = 3;

    // The condition must be a `bool`. No brackets needed around it.
    if number < 5 {
        println!("condition was true");
    } else {
        println!("condition was false");
    }

    // Rust never turns a number into a bool for you. `if number { ... }`
    // is error E0308, "expected `bool`, found integer". Say what you mean:
    if number != 0 {
        println!("number was something other than zero");
    }

    // `else if` checks conditions in order and runs ONLY the first match.
    // 6 is divisible by 3 and by 2, but only the first branch runs.
    // (`is_multiple_of` is a method on unsigned integers, hence the `u32`.)
    let number: u32 = 6;
    if number.is_multiple_of(4) {
        println!("number is divisible by 4");
    } else if number.is_multiple_of(3) {
        println!("number is divisible by 3");
    } else if number.is_multiple_of(2) {
        println!("number is divisible by 2");
    } else {
        println!("number is not divisible by 4, 3, or 2");
    }

    // Conditions combine with && (and), || (or) and ! (not).
    let age = 15;
    let with_adult = true;
    if age >= 18 || with_adult {
        println!("you may watch the film");
    }
}
