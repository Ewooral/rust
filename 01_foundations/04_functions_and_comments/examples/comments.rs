//! Every kind of comment Rust has. This line is an "inner" doc comment: it
//! documents the file (crate or module) it sits in.
//!
//! Run: cargo run -p functions_and_comments --example comments
//! Docs: cargo doc -p functions_and_comments --example comments --document-private-items --open

// A line comment starts with two slashes and runs to the end of the line.
// The compiler ignores it completely. For more lines, start each one with //.

/// A doc comment (three slashes) documents the item right after it.
/// It's written in Markdown, so `code`, *emphasis* and lists all work:
///
/// - It appears in `cargo doc` output and in your editor's hover pop-ups.
/// - It explains what the item does and how to use it, for the people who
///   will call it.
///
/// # Panics
///
/// Never. (Sections like `# Panics`, `# Errors` and `# Examples` are a
/// convention readers look for.)
fn lucky_number() -> u32 {
    // Comments usually go on their own line, ABOVE the code they describe.
    7
}

fn main() {
    let lucky = lucky_number(); // A comment can also end a line of code.

    /* A block comment can span several lines, or sit inside one: */
    let doubled = lucky * /* inline! */ 2;

    // TODO: comments like this mark unfinished work. Many editors list them.
    println!("lucky = {lucky}, doubled = {doubled}");

    // Commenting code out is a quick way to switch it off while you test:
    // println!("this line doesn't run");
}
