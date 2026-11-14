//! The printing macros: `println!`, `print!` and `eprintln!`.
//! Run: cargo run -p hello_world --example print_macros
//!
//! Then hide the error stream and watch the `eprintln!` line disappear:
//!
//! - bash: `cargo run -p hello_world --example print_macros 2>/dev/null`
//! - PowerShell: `cargo run -p hello_world --example print_macros 2>$null`

fn main() {
    // `println!` prints, then moves to a new line.
    println!("println! adds a newline at the end.");

    // `print!` does NOT add a newline, so these three join up on one line.
    print!("print! ");
    print!("stays ");
    print!("on one line");
    // An empty `println!()` just ends the line.
    println!();

    // `eprintln!` prints to STANDARD ERROR, a second output stream meant for
    // problems and diagnostics. On screen it looks the same, but `2>` can
    // send it somewhere else (see the commands at the top).
    eprintln!("eprintln! goes to standard error.");

    // The text is checked at COMPILE time: two `{}` need two values.
    // Uncomment the next line to see "2 positional arguments in format
    // string, but there is 1 argument".
    // println!("{} and {}", "only one");

    // `{{` and `}}` print real braces, and `\n` starts a new line by hand.
    println!("Braces: {{ and }}\nA second line from the same println!");
}
