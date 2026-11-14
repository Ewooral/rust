//! One error type for the inventory, built from `sqlx::Error` and our own checks.

use crate::isbn::IsbnError;
use crate::money::MoneyError;

/// Everything that can go wrong in the bookshop.
#[derive(Debug, thiserror::Error)]
pub enum InventoryError {
    /// The caller passed something invalid (caught before any SQL runs).
    #[error("invalid input: {0}")]
    Invalid(String),
    /// No book with this ISBN.
    #[error("no book with ISBN {0}")]
    BookNotFound(String),
    /// A `UNIQUE` constraint said no.
    #[error("a book with ISBN {0} already exists")]
    DuplicateIsbn(String),
    /// Not enough copies on the shelf. The sale was rolled back.
    #[error("only {available} of {isbn} in stock, cannot sell {wanted}")]
    OutOfStock {
        /// The book.
        isbn: String,
        /// How many the customer wanted.
        wanted: i64,
        /// How many there are.
        available: i64,
    },
    /// A foreign key said no: the book has sales that refer to it.
    #[error("book {0} has been sold, so it can't be deleted")]
    HasSales(String),
    /// Anything else from the database (connection lost, bad SQL, …).
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

impl From<IsbnError> for InventoryError {
    fn from(err: IsbnError) -> Self {
        InventoryError::Invalid(err.to_string())
    }
}

impl From<MoneyError> for InventoryError {
    fn from(err: MoneyError) -> Self {
        InventoryError::Invalid(err.to_string())
    }
}

/// True when `err` is the database rejecting a duplicate in a `UNIQUE` column.
pub fn is_unique_violation(err: &sqlx::Error) -> bool {
    err.as_database_error()
        .is_some_and(|db| db.is_unique_violation())
}

/// True when `err` is a `REFERENCES` (foreign key) constraint failing.
pub fn is_foreign_key_violation(err: &sqlx::Error) -> bool {
    err.as_database_error()
        .is_some_and(|db| db.is_foreign_key_violation())
}
