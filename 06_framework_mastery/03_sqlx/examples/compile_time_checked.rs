//! Compile-time checked queries: `query!`, `query_as!` and `query_scalar!`.
//! Run: cargo run -p sqlx_database --example compile_time_checked
//!
//! These macros check the SQL against a real schema WHILE THE CRATE COMPILES:
//! a misspelt column, a wrong number of `?` arguments or a field of the wrong
//! type is a compile error, not a 3 a.m. production error.
//!
//! To do that the macros need the schema. They get it in one of two ways:
//! - online: `DATABASE_URL` (in the environment or a `.env` file) points at a
//!   migrated database, and the macros ask it to describe each query;
//! - offline: no `DATABASE_URL`, so they read the saved descriptions in the
//!   crate's `.sqlx/` folder, one JSON file per query.
//!
//! This lesson uses offline mode: `.sqlx/` is committed next to `Cargo.toml`,
//! so this example builds anywhere, with no database and no `sqlx-cli`.
//! After changing a query here, regenerate `.sqlx/` (see README.md).

use sqlx_database::{Inventory, db, sample_catalogue};

/// No `FromRow` needed: `query_as!` writes the mapping itself and checks that
/// every field exists in the result with a compatible type.
struct PriceTag {
    title: String,
    price_pence: i64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = db::connect_in_memory().await?;
    Inventory::new(pool.clone())
        .import_catalogue(&sample_catalogue())
        .await?;

    // query!: an anonymous struct with one typed field per column.
    // `published_year` has no NOT NULL, so the macro makes it an Option<i64>.
    let isbn = "9780000000033";
    let row = sqlx::query!(
        "SELECT title, stock, published_year FROM books WHERE isbn = ?",
        isbn
    )
    .fetch_one(&pool)
    .await?;
    let (title, stock, year): (String, i64, Option<i64>) =
        (row.title, row.stock, row.published_year);
    println!("query!        -> {title:?}, {stock} in stock, published {year:?}");

    // query_as!: straight into your own struct.
    let max_price = 800_i64;
    let cheap = sqlx::query_as!(
        PriceTag,
        "SELECT title, price_pence FROM books WHERE price_pence <= ? ORDER BY price_pence",
        max_price
    )
    .fetch_all(&pool)
    .await?;
    for tag in &cheap {
        println!("query_as!     -> {} costs {}p", tag.title, tag.price_pence);
    }

    // query_scalar!: a single column. `AS "count!"` overrides the inferred
    // nullability ("this is never NULL"); `AS "name: Type"` overrides the type.
    let count = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!: i64" FROM books"#)
        .fetch_one(&pool)
        .await?;
    println!("query_scalar! -> {count} books");

    Ok(())
}
