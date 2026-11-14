//! Solution to exercise 2: make it compile (E0277): run a query inside a transaction.
//!
//! `give_star` should add a star to one child and record why, as ONE
//! transaction. But `.execute(&mut tx)` doesn't compile: the methods that run
//! a query take an *executor* (a `&Pool`, a `&mut SqliteConnection`, …), and
//! `&mut Transaction` isn't one. A `Transaction` *derefs* to the connection
//! it wraps, so reborrow that connection instead. Fix both calls.
//!
//! Answer: `&mut *tx` derefs the `Transaction` to the `SqliteConnection`
//! inside it and reborrows that, and `&mut SqliteConnection` IS an executor.
//! `tx` is only borrowed, so it is still there for the next query and for
//! `commit`.
//!
//! Run: cargo run -p sqlx_database --features solutions --bin sol02_fix_transaction_executor

use sqlx::SqlitePool;

async fn give_star(pool: &SqlitePool, child: &str, reason: &str) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE children SET stars = stars + 1 WHERE name = ?")
        .bind(child)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO star_log (name, reason) VALUES (?, ?)")
        .bind(child)
        .bind(reason)
        .execute(&mut *tx)
        .await?;
    tx.commit().await
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    sqlx::raw_sql(
        "CREATE TABLE children (name TEXT PRIMARY KEY, stars INTEGER NOT NULL);
         CREATE TABLE star_log (name TEXT NOT NULL, reason TEXT NOT NULL);
         INSERT INTO children VALUES ('ama', 0), ('kofi', 2);",
    )
    .execute(&pool)
    .await?;

    give_star(&pool, "ama", "tidied her room").await?;
    give_star(&pool, "ama", "fed the cat").await?;
    let stars: Vec<(String, i64)> =
        sqlx::query_as("SELECT name, stars FROM children ORDER BY name")
            .fetch_all(&pool)
            .await?;
    let logged: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM star_log")
        .fetch_one(&pool)
        .await?;
    println!("{stars:?}, {logged} reasons logged");
    Ok(())
}
