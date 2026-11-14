//! Exercise 6: a search with optional filters and pages, using `QueryBuilder`.
//!
//! The table (made by `setup`):
//!
//!   books (id INTEGER PRIMARY KEY, title TEXT NOT NULL,
//!          author TEXT NOT NULL, price_pence INTEGER NOT NULL)
//!
//! Implement `search`: return the matching TITLES, ordered by title, then id.
//! Each filter that is set adds one condition:
//!   - `title_contains`: `title LIKE ?`, binding the value `%text%`
//!   - `max_price`:      `price_pence <= ?`
//!   - `authors` (if not empty): `author IN (?, ?, …)`, one placeholder per
//!     name (`QueryBuilder::separated(", ")` and its `push_bind`)
//!
//! then `LIMIT ? OFFSET ?` from `page` and `per_page` (page 1 is the first).
//!
//! Start from `QueryBuilder::<Sqlite>::new("SELECT title FROM books WHERE 1 = 1")`
//! so every condition can begin with " AND ". The SQL text may only contain
//! YOUR strings; every value from `Filter` goes in with `push_bind`.
//! Finish with `build_query_scalar().fetch_all(pool)`.
//!
//! Test: cargo test -p sqlx_database --features exercises --bin ex06_dynamic_search

#![allow(unused_variables, unused_imports)]

use sqlx::{QueryBuilder, Sqlite, SqlitePool};

#[derive(Debug, Default)]
struct Filter {
    title_contains: Option<String>,
    max_price: Option<i64>,
    authors: Vec<String>,
    /// 1-based page number; 0 counts as 1.
    page: i64,
    /// Rows per page; 0 means 10.
    per_page: i64,
}

async fn search(pool: &SqlitePool, filter: &Filter) -> Result<Vec<String>, sqlx::Error> {
    todo!()
}

async fn setup() -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    sqlx::raw_sql(
        "CREATE TABLE books (id INTEGER PRIMARY KEY, title TEXT NOT NULL,
                             author TEXT NOT NULL, price_pence INTEGER NOT NULL);
         INSERT INTO books (title, author, price_pence) VALUES
            ('Emma', 'Jane Austen', 750),
            ('Persuasion', 'Jane Austen', 650),
            ('Pride and Prejudice', 'Jane Austen', 899),
            ('Dracula', 'Bram Stoker', 899),
            ('Treasure Island', 'Robert Louis Stevenson', 699),
            ('Kidnapped', 'Robert Louis Stevenson', 550),
            ('Frankenstein', 'Mary Shelley', 925);",
    )
    .execute(&pool)
    .await?;
    Ok(pool)
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = setup().await?;
    let filter = Filter {
        max_price: Some(700),
        ..Filter::default()
    };
    println!("{:?}", search(&pool, &filter).await?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn authors(names: &[&str]) -> Vec<String> {
        names.iter().map(|n| n.to_string()).collect()
    }

    #[tokio::test]
    async fn no_filters_returns_the_first_page_by_title() {
        let pool = setup().await.unwrap();
        let all = search(&pool, &Filter::default()).await.unwrap();
        assert_eq!(
            all,
            [
                "Dracula",
                "Emma",
                "Frankenstein",
                "Kidnapped",
                "Persuasion",
                "Pride and Prejudice",
                "Treasure Island"
            ]
        );
    }

    #[tokio::test]
    async fn filters_combine_with_and() {
        let pool = setup().await.unwrap();
        let filter = Filter {
            title_contains: Some("e".into()),
            max_price: Some(750),
            authors: authors(&["Jane Austen", "Robert Louis Stevenson"]),
            ..Filter::default()
        };
        assert_eq!(
            search(&pool, &filter).await.unwrap(),
            ["Emma", "Kidnapped", "Persuasion", "Treasure Island"]
        );
        let one_author = Filter {
            authors: authors(&["Mary Shelley"]),
            ..Filter::default()
        };
        assert_eq!(search(&pool, &one_author).await.unwrap(), ["Frankenstein"]);
    }

    #[tokio::test]
    async fn pages_through_results() {
        let pool = setup().await.unwrap();
        let page = |page| Filter {
            page,
            per_page: 3,
            ..Filter::default()
        };
        assert_eq!(
            search(&pool, &page(1)).await.unwrap(),
            ["Dracula", "Emma", "Frankenstein"]
        );
        assert_eq!(search(&pool, &page(3)).await.unwrap(), ["Treasure Island"]);
        assert!(search(&pool, &page(4)).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn hostile_input_is_only_ever_a_value() {
        let pool = setup().await.unwrap();
        let filter = Filter {
            title_contains: Some("' OR 1=1 --".into()),
            authors: authors(&["x') OR ('1'='1"]),
            ..Filter::default()
        };
        assert!(search(&pool, &filter).await.unwrap().is_empty());
    }
}
