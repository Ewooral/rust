//! Solution to exercise 3: make it compile (E0277).
//!
//! Read the compiler's help text: it tells you the exact line to add.
//! Then change `{:?}` to `{:#?}` and run again. What changed?
//!
//! Run: cargo run -p structs --features solutions --bin sol03_fix_missing_debug

#[derive(Debug)]
struct Book {
    title: String,
    pages: u32,
}

fn main() {
    let book = Book {
        title: String::from("The Rust Programming Language"),
        pages: 560,
    };
    println!("{book:?}");
    println!("{} has {} pages", book.title, book.pages);
}
