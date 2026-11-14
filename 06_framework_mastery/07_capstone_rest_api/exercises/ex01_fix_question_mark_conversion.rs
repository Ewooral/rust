//! Exercise 1: make it compile (E0277): teach `?` to convert between layers.
//!
//! The repository returns `sqlx::Error`. The service returns `AppError`.
//! `?` in the service can only pass a `sqlx::Error` upwards if it knows how
//! to turn it into an `AppError`, which means an `impl From<sqlx::Error> for AppError`.
//!
//! Read the error, then add that `From` impl. A database error the service
//! didn't handle on purpose is our fault, so make it `AppError::Internal`
//! with the error's text. Don't touch `find_title` or `title_of`.
//!
//! Run: cargo run -p capstone_rest_api --features exercises --bin ex01_fix_question_mark_conversion

use sqlx::SqlitePool;

#[derive(Debug)]
enum AppError {
    NotFound(i64),
    Internal(String),
}

/// Repository layer: SQL in, `sqlx::Error` out.
async fn find_title(pool: &SqlitePool, id: i64) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT title FROM books WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Service layer: speaks `AppError`.
async fn title_of(pool: &SqlitePool, id: i64) -> Result<String, AppError> {
    let title = find_title(pool, id).await?;
    title.ok_or(AppError::NotFound(id))
}

fn describe(result: Result<String, AppError>) -> String {
    match result {
        Ok(title) => format!("{title:?}"),
        Err(AppError::NotFound(id)) => format!("404: no book {id}"),
        Err(AppError::Internal(details)) => format!("500: {details}"),
    }
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    sqlx::raw_sql(
        "CREATE TABLE books (id INTEGER PRIMARY KEY, title TEXT NOT NULL);
         INSERT INTO books (title) VALUES ('Emma');",
    )
    .execute(&pool)
    .await?;

    println!("book 1: {}", describe(title_of(&pool, 1).await));
    println!("book 2: {}", describe(title_of(&pool, 2).await));
    sqlx::query("DROP TABLE books").execute(&pool).await?;
    println!("no table: {}", describe(title_of(&pool, 1).await));
    Ok(())
}
