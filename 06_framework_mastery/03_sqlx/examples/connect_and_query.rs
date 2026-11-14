//! A pool, and the `query` family: execute, fetch_one, fetch_all, fetch_optional, fetch.
//! Run: cargo run -p sqlx_database --example connect_and_query

use futures::TryStreamExt;
use sqlx::Row;
use sqlx::sqlite::SqlitePoolOptions;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // A pool of up to 5 connections to a fresh in-memory database.
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite::memory:")
        .await?;

    // `execute`: run a statement, get back how many rows it changed.
    sqlx::query(
        "CREATE TABLE pets (id INTEGER PRIMARY KEY, name TEXT NOT NULL, legs INTEGER NOT NULL)",
    )
    .execute(&pool)
    .await?;
    for (name, legs) in [("Rex", 4), ("Polly", 2), ("Nemo", 0), ("Tiddles", 4)] {
        let done = sqlx::query("INSERT INTO pets (name, legs) VALUES (?, ?)")
            .bind(name)
            .bind(legs)
            .execute(&pool)
            .await?;
        println!(
            "inserted {name}: {} row, id {}",
            done.rows_affected(),
            done.last_insert_rowid()
        );
    }

    // `fetch_one`: exactly one row expected (zero rows is an error).
    let row = sqlx::query("SELECT name, legs FROM pets WHERE id = ?")
        .bind(2)
        .fetch_one(&pool)
        .await?;
    let name: String = row.get("name"); // by column name...
    let legs: i64 = row.get(1); // ...or by position
    println!("fetch_one      -> {name} has {legs} legs");

    // `fetch_all`: every row, collected into a Vec.
    let four_legged = sqlx::query("SELECT name FROM pets WHERE legs = ? ORDER BY name")
        .bind(4)
        .fetch_all(&pool)
        .await?;
    let names: Vec<String> = four_legged.iter().map(|r| r.get("name")).collect();
    println!("fetch_all      -> {names:?}");

    // `fetch_optional`: zero or one row, as an Option.
    let missing = sqlx::query("SELECT name FROM pets WHERE name = ?")
        .bind("Godzilla")
        .fetch_optional(&pool)
        .await?;
    println!("fetch_optional -> found Godzilla? {}", missing.is_some());

    // `query_scalar`: the first column of the row, already decoded.
    let total_legs: i64 = sqlx::query_scalar("SELECT SUM(legs) FROM pets")
        .fetch_one(&pool)
        .await?;
    println!("query_scalar   -> {total_legs} legs altogether");

    // `fetch`: a Stream of rows, one at a time (good for huge results).
    let mut rows = sqlx::query("SELECT name FROM pets ORDER BY id").fetch(&pool);
    print!("fetch (stream) ->");
    while let Some(row) = rows.try_next().await? {
        print!(" {}", row.get::<String, _>("name"));
    }
    println!();
    drop(rows); // the stream was holding a connection: give it back

    // Close the pool politely: waits for borrowed connections to come back.
    pool.close().await;
    Ok(())
}
