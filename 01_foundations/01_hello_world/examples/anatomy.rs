//! The anatomy of "Hello, world!": every piece of the smallest Rust program.
//! Run: cargo run -p hello_world --example anatomy

// `fn` declares a function. `main` is special: every Rust program starts
// running here. The empty `()` means it takes no parameters, and because
// there is no `-> Type`, it gives nothing back.
fn main() {
    // The body sits between `{` and `}`. Rust style is 4 spaces of indent;
    // `cargo fmt` does it for you, so nobody has to argue about it.

    // `println!` ends in `!`, so it is a MACRO, not a function. A macro writes
    // more code for you at compile time: here, the code that formats the text
    // and sends it to the screen. "Hello, world!" is a string literal, and the
    // `;` says "this statement is finished".
    println!("Hello, world!");

    // Most lines of Rust end in `;`, one statement after another, top to bottom.
    println!("Statement one.");
    println!("Statement two.");

    // A preview of chapter 2: `{}` is a placeholder filled in by the values
    // after the string, and `{name}` reads a variable straight into the text.
    let name = "Ferris";
    println!("{} + {} = {}", 2, 2, 2 + 2);
    println!("Hello, {name}!");
}
