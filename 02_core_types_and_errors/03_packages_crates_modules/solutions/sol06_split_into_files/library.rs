//! `crate::library`: its child `books` lives in library/books.rs.

pub mod books;

pub fn name() -> &'static str {
    "City Library"
}
