//! Errors: RowNotFound, constraint violations by kind, and a pool that times out.
//! Run: cargo run -p sqlx_database --example error_handling

use std::time::Duration;

use sqlx::error::ErrorKind;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx_database::{Inventory, InventoryError, Isbn, NewBook, db, sample_catalogue};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = db::connect_in_memory().await?;
    let shop = Inventory::new(pool.clone());
    shop.import_catalogue(&sample_catalogue()).await?;

    // 1. `fetch_one` with no row is an error; `fetch_optional` is an Option.
    let none = sqlx::query("SELECT * FROM books WHERE title = ?")
        .bind("Dracula")
        .fetch_one(&pool)
        .await;
    println!("fetch_one, no row:      {:?}", none.unwrap_err());
    let maybe = sqlx::query("SELECT * FROM books WHERE title = ?")
        .bind("Dracula")
        .fetch_optional(&pool)
        .await?;
    println!("fetch_optional, no row: {:?}", maybe.map(|_| "a row"));

    // 2. Constraint violations are `sqlx::Error::Database`. Ask the error
    //    what KIND it is, instead of matching on message text.
    let attempts = [
        (
            "duplicate ISBN",
            "INSERT INTO books (isbn, title, author_id, price_pence) VALUES ('9780000000019', 'Again', 1, 100)",
        ),
        (
            "unknown author",
            "INSERT INTO books (isbn, title, author_id, price_pence) VALUES ('9780000000064', 'Orphan', 99, 100)",
        ),
        (
            "negative price",
            "INSERT INTO books (isbn, title, author_id, price_pence) VALUES ('9780000000071', 'Cheap', 1, -1)",
        ),
        (
            "missing title",
            "INSERT INTO books (isbn, author_id, price_pence) VALUES ('9780000000088', 1, 100)",
        ),
    ];
    for (what, sql) in attempts {
        let err = sqlx::query(sql).execute(&pool).await.unwrap_err();
        let db_err = err.as_database_error().expect("a database error");
        let kind = match db_err.kind() {
            ErrorKind::UniqueViolation => "UniqueViolation",
            ErrorKind::ForeignKeyViolation => "ForeignKeyViolation",
            ErrorKind::CheckViolation => "CheckViolation",
            ErrorKind::NotNullViolation => "NotNullViolation",
            _ => "Other",
        };
        println!(
            "{what:<15} -> {kind:<19} code {:?}: {}",
            db_err.code(),
            db_err.message()
        );
    }

    // 3. The repository turns those into errors the bookshop understands.
    let again = NewBook::parse("9780000000019", "Again", "Jane Austen", "1.00", 1)?;
    println!(
        "\nadd_book(duplicate):  {}",
        shop.add_book(&again).await.unwrap_err()
    );
    let emma: Isbn = "9780000000026".parse()?;
    shop.sell(&emma, 1).await?;
    println!(
        "remove_book(sold):    {}",
        shop.remove_book(&emma).await.unwrap_err()
    );
    match shop.sell(&emma, 5).await {
        Err(InventoryError::OutOfStock { available, .. }) => {
            println!("sell(5):              only {available} left")
        }
        other => println!("unexpected: {other:?}"),
    }
    println!(
        "parse(bad ISBN):      {}",
        NewBook::parse("978-0", "T", "A", "1", 1).unwrap_err()
    );

    // 4. A pool with one connection, already lent out: the next `acquire`
    //    waits `acquire_timeout`, then gives up with PoolTimedOut.
    let tiny = SqlitePoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_millis(200))
        .connect("sqlite::memory:")
        .await?;
    let _held = tiny.acquire().await?;
    let err = tiny.acquire().await.unwrap_err();
    println!("\nsecond acquire:       {err} ({err:?})");
    Ok(())
}
