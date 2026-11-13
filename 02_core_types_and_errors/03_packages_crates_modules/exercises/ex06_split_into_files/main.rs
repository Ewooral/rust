//! Exercise 6: make it compile (E0583) by creating module files.
//!
//! This program is a folder: exercises/ex06_split_into_files/. `main.rs`
//! declares `mod library;`, but the file it points to does not exist yet.
//!
//! Create the files so the module tree is:
//!   crate
//!   └── library            -> which file?
//!       └── books          -> which file?
//!
//! Put this in `library`:
//!
//!     pub mod books;
//!
//!     pub fn name() -> &'static str {
//!         "City Library"
//!     }
//!
//! Put this in `library::books`:
//!
//!     pub struct Book {
//!         title: String,
//!         pages: u32,
//!     }
//!
//!     impl Book {
//!         pub fn new(title: &str, pages: u32) -> Self {
//!             Self { title: title.to_string(), pages }
//!         }
//!
//!         pub fn summary(&self) -> String {
//!             format!("{} ({} pages)", self.title, self.pages)
//!         }
//!     }
//!
//! Hint: the compiler's error message tells you the file names it tried.
//!
//! Run: cargo run -p packages_crates_modules --features exercises --bin ex06_split_into_files

// `cargo fmt` cannot read a module file that does not exist yet, so this
// line tells it to skip the module. Delete it once your files exist.
#[rustfmt::skip]
mod library;

use library::books::Book;

fn main() {
    let book = Book::new("The Rust Programming Language", 560);
    println!("{} has {}", library::name(), book.summary());
}
