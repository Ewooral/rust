//! The bookshop's repository: every SQL statement the program runs lives here.
//!
//! Each method borrows a connection from the pool, runs bound queries, and
//! turns rows into Rust types. Methods that make several changes that must
//! succeed or fail *together* use a transaction.

use sqlx::{QueryBuilder, Sqlite, SqliteConnection, SqlitePool};

use crate::error::{InventoryError, is_foreign_key_violation, is_unique_violation};
use crate::isbn::Isbn;
use crate::models::{Book, BookFilter, NewBook, Sale, SalesLine};
use crate::money::Pence;

/// The columns of a [`Book`], joined with its author, for every query that
/// returns books. It's a macro rather than a `const` so that `concat!` can
/// glue it to more SQL **at compile time**: the result is still a
/// `&'static str`, which is what `sqlx::query*` accept without complaint.
macro_rules! select_books {
    () => {
        "SELECT b.id, b.isbn, b.title, a.name AS author, \
         b.price_pence, b.stock, b.published_year \
         FROM books b JOIN authors a ON a.id = b.author_id"
    };
}

/// The bookshop's stock, sales and catalogue.
#[derive(Debug, Clone)]
pub struct Inventory {
    pool: SqlitePool,
}

impl Inventory {
    /// Wraps a pool whose database has been migrated (see [`crate::db`]).
    ///
    /// Cloning an `Inventory` (or a `SqlitePool`) is cheap: the clones share
    /// the same connections.
    pub fn new(pool: SqlitePool) -> Self {
        Inventory { pool }
    }

    /// The pool, for callers that want to run their own queries.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Adds one book (and its author, if new) in a single transaction.
    pub async fn add_book(&self, new: &NewBook) -> Result<Book, InventoryError> {
        let mut tx = self.pool.begin().await?;
        insert_book(&mut tx, new).await?;
        tx.commit().await?;
        self.book(&new.isbn).await
    }

    /// Adds a whole catalogue, all or nothing: if any book fails, none are added.
    pub async fn import_catalogue(&self, books: &[NewBook]) -> Result<usize, InventoryError> {
        let mut tx = self.pool.begin().await?;
        for new in books {
            // `?` returns early on the first error; `tx` is then dropped
            // without `commit`, and dropping it rolls everything back.
            insert_book(&mut tx, new).await?;
        }
        tx.commit().await?;
        Ok(books.len())
    }

    /// The book with this ISBN, or `None`: `fetch_optional`.
    pub async fn find_book(&self, isbn: &Isbn) -> Result<Option<Book>, InventoryError> {
        let book = sqlx::query_as::<_, Book>(concat!(select_books!(), " WHERE b.isbn = ?"))
            .bind(isbn)
            .fetch_optional(&self.pool)
            .await?;
        Ok(book)
    }

    /// The book with this ISBN, or [`InventoryError::BookNotFound`].
    pub async fn book(&self, isbn: &Isbn) -> Result<Book, InventoryError> {
        self.find_book(isbn)
            .await?
            .ok_or_else(|| InventoryError::BookNotFound(isbn.to_string()))
    }

    /// Every book, by title: `fetch_all`.
    pub async fn books(&self) -> Result<Vec<Book>, InventoryError> {
        let sql = concat!(select_books!(), " ORDER BY b.title");
        Ok(sqlx::query_as(sql).fetch_all(&self.pool).await?)
    }

    /// How many different books: a single value, so `query_scalar`.
    pub async fn count_books(&self) -> Result<i64, InventoryError> {
        let count = sqlx::query_scalar("SELECT COUNT(*) FROM books")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }

    /// The shelf value: price × stock, summed. `COALESCE` turns the `NULL`
    /// that `SUM` returns for an empty table into 0.
    pub async fn stock_value(&self) -> Result<Pence, InventoryError> {
        let total = sqlx::query_scalar("SELECT COALESCE(SUM(price_pence * stock), 0) FROM books")
            .fetch_one(&self.pool)
            .await?;
        Ok(total)
    }

    /// Searches with optional filters and a page, building the SQL at run time
    /// with [`QueryBuilder`]: SQL text from *our* code, values always bound.
    pub async fn search(&self, filter: &BookFilter) -> Result<Vec<Book>, InventoryError> {
        let mut query = QueryBuilder::<Sqlite>::new(select_books!());
        query.push(" WHERE 1 = 1");
        if let Some(text) = &filter.title_contains {
            // The `%` wildcards go in the bound *value*, never in the SQL.
            query
                .push(" AND b.title LIKE ")
                .push_bind(format!("%{text}%"));
        }
        if let Some(author) = &filter.author {
            query.push(" AND a.name = ").push_bind(author.clone());
        }
        if let Some(max) = filter.max_price {
            query.push(" AND b.price_pence <= ").push_bind(max);
        }
        if filter.in_stock_only {
            query.push(" AND b.stock > 0");
        }
        let limit = filter.limit.unwrap_or(50).clamp(1, 500);
        query
            .push(" ORDER BY b.title, b.id LIMIT ")
            .push_bind(limit)
            .push(" OFFSET ")
            .push_bind(filter.offset.max(0));
        Ok(query.build_query_as::<Book>().fetch_all(&self.pool).await?)
    }

    /// Puts more copies on the shelf and returns the new stock level.
    pub async fn restock(&self, isbn: &Isbn, copies: i64) -> Result<i64, InventoryError> {
        if copies <= 0 {
            return Err(InventoryError::Invalid(format!(
                "restock needs a positive number of copies, not {copies}"
            )));
        }
        // `RETURNING` hands back the updated value in the same statement.
        sqlx::query_scalar("UPDATE books SET stock = stock + ? WHERE isbn = ? RETURNING stock")
            .bind(copies)
            .bind(isbn)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| InventoryError::BookNotFound(isbn.to_string()))
    }

    /// Sells copies: takes them off the shelf **and** records the sale, in one
    /// transaction. Either both happen or neither does.
    pub async fn sell(&self, isbn: &Isbn, quantity: i64) -> Result<Sale, InventoryError> {
        if quantity <= 0 {
            return Err(InventoryError::Invalid(format!(
                "can't sell {quantity} copies"
            )));
        }
        let mut tx = self.pool.begin().await?;

        // 1. Check and update in ONE statement: "take `quantity` copies, but
        //    only if there are that many". Reading the stock first and writing
        //    it back later would let two tills sell the last copy twice.
        let taken: Option<(i64, Pence, i64)> = sqlx::query_as(
            "UPDATE books SET stock = stock - ? \
             WHERE isbn = ? AND stock >= ? \
             RETURNING id, price_pence, stock",
        )
        .bind(quantity)
        .bind(isbn)
        .bind(quantity)
        .fetch_optional(&mut *tx)
        .await?;

        let Some((book_id, price, stock_left)) = taken else {
            // Find out why nothing was updated. Returning here drops `tx`,
            // which rolls the transaction back.
            let available: Option<i64> =
                sqlx::query_scalar("SELECT stock FROM books WHERE isbn = ?")
                    .bind(isbn)
                    .fetch_optional(&mut *tx)
                    .await?;
            return Err(match available {
                None => InventoryError::BookNotFound(isbn.to_string()),
                Some(available) => InventoryError::OutOfStock {
                    isbn: isbn.to_string(),
                    wanted: quantity,
                    available,
                },
            });
        };

        let total = price
            .times(quantity)
            .ok_or_else(|| InventoryError::Invalid("the total is too large".into()))?;

        // 2. Record the sale. If this fails, step 1 is undone too.
        let sale_id: i64 = sqlx::query_scalar(
            "INSERT INTO sales (book_id, quantity, total_pence) VALUES (?, ?, ?) RETURNING id",
        )
        .bind(book_id)
        .bind(quantity)
        .bind(total)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(Sale {
            sale_id,
            isbn: isbn.clone(),
            quantity,
            total,
            stock_left,
        })
    }

    /// Copies sold and money taken per book, best sellers first.
    pub async fn sales_report(&self) -> Result<Vec<SalesLine>, InventoryError> {
        let lines = sqlx::query_as::<_, SalesLine>(
            "SELECT b.title, SUM(s.quantity) AS copies_sold, SUM(s.total_pence) AS revenue_pence \
             FROM sales s JOIN books b ON b.id = s.book_id \
             GROUP BY b.id \
             ORDER BY revenue_pence DESC, b.title",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(lines)
    }

    /// Deletes a book that has never been sold.
    pub async fn remove_book(&self, isbn: &Isbn) -> Result<(), InventoryError> {
        let result = sqlx::query("DELETE FROM books WHERE isbn = ?")
            .bind(isbn)
            .execute(&self.pool)
            .await;
        match result {
            // `execute` reports how many rows changed; 0 means "no such book".
            Ok(done) if done.rows_affected() == 0 => {
                Err(InventoryError::BookNotFound(isbn.to_string()))
            }
            Ok(_) => Ok(()),
            Err(err) if is_foreign_key_violation(&err) => {
                Err(InventoryError::HasSales(isbn.to_string()))
            }
            Err(err) => Err(err.into()),
        }
    }
}

/// Inserts one book on a connection that the caller provides: a transaction
/// (`&mut *tx` derefs to `&mut SqliteConnection`), or a plain connection.
async fn insert_book(conn: &mut SqliteConnection, new: &NewBook) -> Result<i64, InventoryError> {
    // Find or create the author. `ON CONFLICT DO NOTHING` skips the insert
    // when the name is already there; either way we then read the id.
    sqlx::query("INSERT INTO authors (name) VALUES (?) ON CONFLICT (name) DO NOTHING")
        .bind(&new.author)
        .execute(&mut *conn)
        .await?;
    let author_id: i64 = sqlx::query_scalar("SELECT id FROM authors WHERE name = ?")
        .bind(&new.author)
        .fetch_one(&mut *conn)
        .await?;

    let inserted = sqlx::query_scalar(
        "INSERT INTO books (isbn, title, author_id, price_pence, stock, published_year) \
         VALUES (?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(&new.isbn)
    .bind(&new.title)
    .bind(author_id)
    .bind(new.price)
    .bind(new.stock)
    .bind(new.published_year)
    .fetch_one(&mut *conn)
    .await;

    match inserted {
        Ok(id) => Ok(id),
        Err(err) if is_unique_violation(&err) => {
            Err(InventoryError::DuplicateIsbn(new.isbn.to_string()))
        }
        Err(err) => Err(err.into()),
    }
}
