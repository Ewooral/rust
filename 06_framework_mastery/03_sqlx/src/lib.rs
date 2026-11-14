//! # A bookshop inventory on SQLx and SQLite
//!
//! The lesson's mini project. Read it in this order:
//!
//! 1. [`money`] and [`isbn`]: validation in plain Rust, tested with no database.
//! 2. `migrations/` (next to `Cargo.toml`): the schema, one numbered file per change.
//! 3. [`db`]: a connection pool, and the embedded migrations (`sqlx::migrate!`).
//! 4. [`models`]: rows as structs, with `#[derive(sqlx::FromRow)]`.
//! 5. [`inventory`]: every query, with bind parameters and transactions.
//! 6. [`error`]: turning `sqlx::Error` into errors the bookshop understands.
//!
//! ```no_run
//! # async fn demo() -> Result<(), sqlx_database::InventoryError> {
//! use sqlx_database::{Inventory, NewBook, db};
//!
//! let pool = db::connect_in_memory().await?; // migrated and empty
//! let shop = Inventory::new(pool);
//! let book = NewBook::parse("978-1-7185-0310-6", "The Rust Programming Language",
//!                           "Steve Klabnik", "39.99", 3)?;
//! shop.add_book(&book).await?;
//! let sale = shop.sell(&book.isbn, 2).await?;
//! assert_eq!(sale.stock_left, 1);
//! # Ok(())
//! # }
//! ```

pub mod db;
pub mod error;
pub mod inventory;
pub mod isbn;
pub mod models;
pub mod money;

pub use error::InventoryError;
pub use inventory::Inventory;
pub use isbn::Isbn;
pub use models::{Book, BookFilter, NewBook, Sale, SalesLine};
pub use money::Pence;

/// A few example books, used by the demo, the examples and the tests.
///
/// The ISBNs are made up (but valid: their check digits are right).
pub fn sample_catalogue() -> Vec<NewBook> {
    const BOOKS: [(&str, &str, &str, &str, i64, i64); 5] = [
        (
            "978-0-00-000001-9",
            "Pride and Prejudice",
            "Jane Austen",
            "8.99",
            4,
            1813,
        ),
        ("978-0-00-000002-6", "Emma", "Jane Austen", "7.50", 2, 1815),
        (
            "978-0-00-000003-3",
            "Treasure Island",
            "Robert Louis Stevenson",
            "6.99",
            5,
            1883,
        ),
        (
            "978-0-00-000004-0",
            "Frankenstein",
            "Mary Shelley",
            "9.25",
            1,
            1818,
        ),
        (
            "978-0-00-000005-7",
            "Alice's Adventures in Wonderland",
            "Lewis Carroll",
            "5.99",
            0,
            1865,
        ),
    ];
    BOOKS
        .iter()
        .map(|&(isbn, title, author, price, stock, year)| {
            NewBook::parse(isbn, title, author, price, stock)
                .expect("sample data is valid")
                .published(year)
        })
        .collect()
}
