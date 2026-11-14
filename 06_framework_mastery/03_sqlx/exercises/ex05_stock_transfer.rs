//! Exercise 5: a transaction that moves stock between two branches.
//!
//! The table (made by `setup`):
//!
//!   stock (branch TEXT, isbn TEXT, copies INTEGER NOT NULL CHECK (copies >= 0),
//!          PRIMARY KEY (branch, isbn))
//!
//! Implement `transfer` so that copies leave one branch and arrive at another
//! **together or not at all**:
//!   1. Refuse `from == to` with `TransferError::SameBranch`, and a count that
//!      isn't positive with `TransferError::BadCount`, before touching the DB.
//!   2. `pool.begin()`.
//!   3. Take the copies with ONE conditional statement:
//!      `UPDATE stock SET copies = copies - ? WHERE branch = ? AND isbn = ? AND copies >= ?`
//!      If `rows_affected()` is 0 there weren't enough: look up how many there
//!      are (0 if there's no row) and return `TransferError::NotEnough`.
//!      Returning drops the transaction, which rolls it back.
//!   4. Add them at the other branch, creating the row if it's missing:
//!      `INSERT INTO stock (branch, isbn, copies) VALUES (?, ?, ?)
//!       ON CONFLICT (branch, isbn) DO UPDATE SET copies = copies + excluded.copies`
//!   5. `commit`.
//!
//! Remember: inside a transaction, run queries on `&mut *tx`.
//!
//! Test: cargo test -p sqlx_database --features exercises --bin ex05_stock_transfer

#![allow(unused_variables, dead_code)]

use sqlx::SqlitePool;

#[derive(Debug, thiserror::Error)]
enum TransferError {
    #[error("a transfer needs two different branches")]
    SameBranch,
    #[error("can't transfer {0} copies")]
    BadCount(i64),
    #[error("only {available} copies available")]
    NotEnough { available: i64 },
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

async fn transfer(
    pool: &SqlitePool,
    isbn: &str,
    from: &str,
    to: &str,
    copies: i64,
) -> Result<(), TransferError> {
    todo!()
}

/// How many copies a branch has (0 if it has no row for the book).
async fn copies_at(pool: &SqlitePool, branch: &str, isbn: &str) -> Result<i64, sqlx::Error> {
    let copies: Option<i64> =
        sqlx::query_scalar("SELECT copies FROM stock WHERE branch = ? AND isbn = ?")
            .bind(branch)
            .bind(isbn)
            .fetch_optional(pool)
            .await?;
    Ok(copies.unwrap_or(0))
}

async fn setup() -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    sqlx::raw_sql(
        "CREATE TABLE stock (branch TEXT NOT NULL, isbn TEXT NOT NULL,
                             copies INTEGER NOT NULL CHECK (copies >= 0),
                             PRIMARY KEY (branch, isbn));
         INSERT INTO stock VALUES ('accra', '9780000000019', 5),
                                  ('kumasi', '9780000000019', 1);",
    )
    .execute(&pool)
    .await?;
    Ok(pool)
}

#[tokio::main]
async fn main() -> Result<(), TransferError> {
    let pool = setup().await?;
    transfer(&pool, "9780000000019", "accra", "kumasi", 2).await?;
    for branch in ["accra", "kumasi"] {
        let copies = copies_at(&pool, branch, "9780000000019").await?;
        println!("{branch}: {copies}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ISBN: &str = "9780000000019";

    #[tokio::test]
    async fn moves_copies_between_branches() {
        let pool = setup().await.unwrap();
        transfer(&pool, ISBN, "accra", "kumasi", 2).await.unwrap();
        assert_eq!(copies_at(&pool, "accra", ISBN).await.unwrap(), 3);
        assert_eq!(copies_at(&pool, "kumasi", ISBN).await.unwrap(), 3);
    }

    #[tokio::test]
    async fn creates_the_row_at_a_new_branch() {
        let pool = setup().await.unwrap();
        transfer(&pool, ISBN, "accra", "tamale", 5).await.unwrap();
        assert_eq!(copies_at(&pool, "accra", ISBN).await.unwrap(), 0);
        assert_eq!(copies_at(&pool, "tamale", ISBN).await.unwrap(), 5);
    }

    #[tokio::test]
    async fn not_enough_changes_nothing() {
        let pool = setup().await.unwrap();
        let err = transfer(&pool, ISBN, "kumasi", "accra", 4)
            .await
            .unwrap_err();
        assert!(
            matches!(err, TransferError::NotEnough { available: 1 }),
            "{err:?}"
        );
        let err = transfer(&pool, ISBN, "tamale", "accra", 1)
            .await
            .unwrap_err();
        assert!(
            matches!(err, TransferError::NotEnough { available: 0 }),
            "{err:?}"
        );
        assert_eq!(copies_at(&pool, "accra", ISBN).await.unwrap(), 5);
        assert_eq!(copies_at(&pool, "kumasi", ISBN).await.unwrap(), 1);
    }

    #[tokio::test]
    async fn refuses_nonsense_before_touching_the_database() {
        let pool = setup().await.unwrap();
        let same = transfer(&pool, ISBN, "accra", "accra", 1)
            .await
            .unwrap_err();
        assert!(matches!(same, TransferError::SameBranch));
        let zero = transfer(&pool, ISBN, "accra", "kumasi", 0)
            .await
            .unwrap_err();
        assert!(matches!(zero, TransferError::BadCount(0)));
    }
}
