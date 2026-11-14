//! `#[sqlx::test]`: SQLx's own test attribute.
//!
//! It runs the test on a Tokio runtime and hands it a pool connected to a
//! **fresh database per test**, with `./migrations` already applied. For
//! SQLite that database is a file under `target/sqlx/test-dbs/`, deleted when
//! the test passes; for Postgres and MySQL it creates (and drops) a whole
//! database on the server named by `DATABASE_URL`.

use sqlx::SqlitePool;
use sqlx_database::{Inventory, Pence, sample_catalogue};

#[sqlx::test]
async fn starts_with_the_migrated_empty_schema(pool: SqlitePool) {
    let books: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM books")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(books, 0);
}

#[sqlx::test]
async fn each_test_gets_its_own_database(pool: SqlitePool) {
    let shop = Inventory::new(pool);
    shop.import_catalogue(&sample_catalogue()).await.unwrap();
    assert_eq!(shop.count_books().await.unwrap(), 5);
    assert_eq!(shop.stock_value().await.unwrap(), Pence(9516));
}
