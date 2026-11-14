//! Solution to exercise 4: create, read, update and count with bound queries.
//!
//! The table (made by `setup`):
//!
//!   books (id INTEGER PRIMARY KEY, title TEXT NOT NULL,
//!          author TEXT NOT NULL, price_pence INTEGER NOT NULL)
//!
//! Implement the five functions. Every value goes in with `.bind(...)`.
//!   - `add_book`:        INSERT, and return the new id (`RETURNING id`, or
//!     `last_insert_rowid()` on the result of `execute`).
//!   - `find_by_title`:   `query_as` into `Book` (derive `sqlx::FromRow` on
//!     it); `fetch_optional`, so a missing title is `Ok(None)`.
//!   - `cheaper_than`:    books with price_pence < max, cheapest first.
//!   - `count_by_author`: a single number, so `query_scalar`.
//!   - `set_price`:       UPDATE; return `true` if a row changed
//!     (`rows_affected()`), `false` for an unknown id.
//!
//! Test: cargo test -p sqlx_database --features solutions --bin sol04_crud_queries

use sqlx::SqlitePool;

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
struct Book {
    id: i64,
    title: String,
    author: String,
    price_pence: i64,
}

async fn add_book(
    pool: &SqlitePool,
    title: &str,
    author: &str,
    price_pence: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO books (title, author, price_pence) VALUES (?, ?, ?) RETURNING id",
    )
    .bind(title)
    .bind(author)
    .bind(price_pence)
    .fetch_one(pool)
    .await
}

async fn find_by_title(pool: &SqlitePool, title: &str) -> Result<Option<Book>, sqlx::Error> {
    sqlx::query_as("SELECT id, title, author, price_pence FROM books WHERE title = ?")
        .bind(title)
        .fetch_optional(pool)
        .await
}

async fn cheaper_than(pool: &SqlitePool, max_pence: i64) -> Result<Vec<Book>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, title, author, price_pence FROM books          WHERE price_pence < ? ORDER BY price_pence, id",
    )
    .bind(max_pence)
    .fetch_all(pool)
    .await
}

async fn count_by_author(pool: &SqlitePool, author: &str) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(*) FROM books WHERE author = ?")
        .bind(author)
        .fetch_one(pool)
        .await
}

async fn set_price(pool: &SqlitePool, id: i64, price_pence: i64) -> Result<bool, sqlx::Error> {
    let done = sqlx::query("UPDATE books SET price_pence = ? WHERE id = ?")
        .bind(price_pence)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(done.rows_affected() == 1)
}

async fn setup() -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    sqlx::query(
        "CREATE TABLE books (id INTEGER PRIMARY KEY, title TEXT NOT NULL,
                             author TEXT NOT NULL, price_pence INTEGER NOT NULL)",
    )
    .execute(&pool)
    .await?;
    Ok(pool)
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = setup().await?;
    let id = add_book(&pool, "Emma", "Jane Austen", 750).await?;
    add_book(&pool, "Dracula", "Bram Stoker", 899).await?;
    println!("added Emma as id {id}");
    println!("{:?}", find_by_title(&pool, "Emma").await?);
    println!("changed: {}", set_price(&pool, id, 699).await?);
    println!("under £8: {:?}", cheaper_than(&pool, 800).await?);
    println!(
        "by Austen: {}",
        count_by_author(&pool, "Jane Austen").await?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn seeded() -> SqlitePool {
        let pool = setup().await.unwrap();
        add_book(&pool, "Emma", "Jane Austen", 750).await.unwrap();
        add_book(&pool, "Persuasion", "Jane Austen", 650)
            .await
            .unwrap();
        add_book(&pool, "Dracula", "Bram Stoker", 899)
            .await
            .unwrap();
        pool
    }

    #[tokio::test]
    async fn adds_and_finds_books() {
        let pool = setup().await.unwrap();
        let first = add_book(&pool, "Emma", "Jane Austen", 750).await.unwrap();
        let second = add_book(&pool, "Dracula", "Bram Stoker", 899)
            .await
            .unwrap();
        assert_ne!(first, second);
        let emma = find_by_title(&pool, "Emma").await.unwrap();
        assert_eq!(
            emma,
            Some(Book {
                id: first,
                title: "Emma".into(),
                author: "Jane Austen".into(),
                price_pence: 750,
            })
        );
        assert_eq!(find_by_title(&pool, "Ulysses").await.unwrap(), None);
    }

    #[tokio::test]
    async fn filters_and_counts() {
        let pool = seeded().await;
        let cheap: Vec<String> = cheaper_than(&pool, 800)
            .await
            .unwrap()
            .into_iter()
            .map(|b| b.title)
            .collect();
        assert_eq!(cheap, ["Persuasion", "Emma"]);
        assert!(cheaper_than(&pool, 100).await.unwrap().is_empty());
        assert_eq!(count_by_author(&pool, "Jane Austen").await.unwrap(), 2);
        assert_eq!(count_by_author(&pool, "Nobody").await.unwrap(), 0);
    }

    #[tokio::test]
    async fn updates_prices() {
        let pool = seeded().await;
        let dracula = find_by_title(&pool, "Dracula").await.unwrap().unwrap();
        assert!(set_price(&pool, dracula.id, 499).await.unwrap());
        let dracula = find_by_title(&pool, "Dracula").await.unwrap().unwrap();
        assert_eq!(dracula.price_pence, 499);
        assert!(!set_price(&pool, 9999, 1).await.unwrap());
    }

    #[tokio::test]
    async fn quotes_in_values_are_just_text() {
        let pool = setup().await.unwrap();
        add_book(&pool, "Alice's Adventures", "Lewis Carroll", 599)
            .await
            .unwrap();
        let alice = find_by_title(&pool, "Alice's Adventures").await.unwrap();
        assert_eq!(alice.map(|b| b.author), Some("Lewis Carroll".to_string()));
        assert_eq!(find_by_title(&pool, "x' OR '1'='1").await.unwrap(), None);
    }
}
