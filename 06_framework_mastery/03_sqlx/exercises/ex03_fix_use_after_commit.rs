//! Exercise 3: make it compile (E0382): nothing runs after `commit`.
//!
//! `rename_book` changes a title and writes an audit row saying so. The audit
//! row is written AFTER `tx.commit()`, which doesn't compile: `commit` takes
//! the transaction by value (`self`), so `tx` is gone.
//!
//! Don't "fix" it by running the audit insert on the pool instead: then the
//! rename and its audit row could be split by a crash in between, which is
//! exactly what the transaction is for. Move the audit insert so that both
//! statements are inside the transaction, and commit once at the end.
//!
//! Run: cargo run -p sqlx_database --features exercises --bin ex03_fix_use_after_commit

use sqlx::SqlitePool;

async fn rename_book(pool: &SqlitePool, id: i64, new_title: &str) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE books SET title = ? WHERE id = ?")
        .bind(new_title)
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    sqlx::query("INSERT INTO audit (book_id, change) VALUES (?, ?)")
        .bind(id)
        .bind(format!("renamed to {new_title}"))
        .execute(&mut *tx)
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    sqlx::raw_sql(
        "CREATE TABLE books (id INTEGER PRIMARY KEY, title TEXT NOT NULL);
         CREATE TABLE audit (book_id INTEGER NOT NULL, change TEXT NOT NULL);
         INSERT INTO books (title) VALUES ('Treasure Isle');",
    )
    .execute(&pool)
    .await?;

    rename_book(&pool, 1, "Treasure Island").await?;
    let title: String = sqlx::query_scalar("SELECT title FROM books WHERE id = 1")
        .fetch_one(&pool)
        .await?;
    let audit: Vec<String> = sqlx::query_scalar("SELECT change FROM audit")
        .fetch_all(&pool)
        .await?;
    println!("title: {title:?}, audit: {audit:?}");
    Ok(())
}
