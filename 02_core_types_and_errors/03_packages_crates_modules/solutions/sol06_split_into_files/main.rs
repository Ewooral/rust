//! Solution to exercise 6. The files:
//!   solutions/sol06_split_into_files/main.rs            crate root
//!   solutions/sol06_split_into_files/library.rs         mod library
//!   solutions/sol06_split_into_files/library/books.rs   mod library::books
//! (`library/mod.rs` instead of `library.rs` works too.)
//!
//! Run: cargo run -p packages_crates_modules --features solutions --bin sol06_split_into_files

mod library;

use library::books::Book;

fn main() {
    let book = Book::new("The Rust Programming Language", 560);
    println!("{} has {}", library::name(), book.summary());
}
