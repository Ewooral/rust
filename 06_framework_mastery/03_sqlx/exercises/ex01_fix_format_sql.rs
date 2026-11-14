//! Exercise 1: make it compile (E0277): don't build SQL with `format!`.
//!
//! `find_secret` glues the user's input into the SQL text. That is SQL
//! injection: the input `nobody' OR '1'='1` turns the query into
//! `... WHERE name = 'nobody' OR '1'='1'`, which matches every row.
//!
//! SQLx 0.9 won't take a `String` as SQL: `query*` functions accept
//! `&'static str` (SQL written in your code) and refuse text built at run
//! time. Read the error message, then fix `find_secret` the right way: a `?`
//! placeholder in a literal SQL string, and `.bind(name)`.
//! (Wrapping the string in `sqlx::AssertSqlSafe` also compiles, but it is a
//! promise that you checked the string yourself, and here it would be a lie.)
//!
//! When it works, `main` prints the secret for "ama" and `None` for the attack.
//!
//! Run: cargo run -p sqlx_database --features exercises --bin ex01_fix_format_sql

use sqlx::SqlitePool;

async fn find_secret(pool: &SqlitePool, name: &str) -> Result<Option<String>, sqlx::Error> {
    let sql = format!("SELECT secret FROM users WHERE name = '{name}'");
    sqlx::query_scalar(sql).fetch_optional(pool).await
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    sqlx::raw_sql(
        "CREATE TABLE users (name TEXT NOT NULL, secret TEXT NOT NULL);
         INSERT INTO users VALUES ('ama', 'likes jollof'), ('kofi', 'afraid of geese');",
    )
    .execute(&pool)
    .await?;

    for name in ["ama", "nobody' OR '1'='1"] {
        println!("{name:?} -> {:?}", find_secret(&pool, name).await?);
    }
    Ok(())
}
