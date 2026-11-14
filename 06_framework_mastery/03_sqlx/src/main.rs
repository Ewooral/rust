//! The bookshop inventory: migrations, queries and transactions on SQLite.
//!
//! Run:  cargo run -p sqlx_database                                 (in memory)
//!       cargo run -p sqlx_database -- "sqlite:bookshop.db?mode=rwc"  (a file that persists)
//!
//! With a file, run it twice: the second run finds the migrations already
//! applied and the books already there, and keeps selling.

use sqlx_database::{BookFilter, Inventory, InventoryError, Isbn, Pence, db, sample_catalogue};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "sqlite::memory:".to_string());
    println!("== database: {url}");

    // 1. Connect (a pool) and bring the schema up to date.
    let pool = db::connect(&url).await?;
    db::MIGRATOR.run(&pool).await?;
    let applied: Vec<(i64, String)> =
        sqlx::query_as("SELECT version, description FROM _sqlx_migrations ORDER BY version")
            .fetch_all(&pool)
            .await?;
    for (version, description) in &applied {
        println!("migration {version}: {description}");
    }

    let shop = Inventory::new(pool);

    // 2. Seed an empty database, all or nothing.
    if shop.count_books().await? == 0 {
        let added = shop.import_catalogue(&sample_catalogue()).await?;
        println!("\n== imported {added} books in one transaction");
    }

    // 3. Read: every book, then a filtered search.
    println!("\n== the shelf");
    for book in shop.books().await? {
        println!(
            "{} {:<34} {:<24} {:>7} x{}",
            book.isbn,
            book.title,
            book.author,
            book.price.to_string(),
            book.stock
        );
    }
    let cheap = BookFilter {
        max_price: Some(Pence(800)),
        in_stock_only: true,
        ..BookFilter::default()
    };
    let titles: Vec<String> = shop
        .search(&cheap)
        .await?
        .into_iter()
        .map(|b| b.title)
        .collect();
    println!("in stock and at most £8.00: {titles:?}");

    // 4. Write: a sale is two changes in one transaction.
    println!("\n== the till");
    let emma: Isbn = "978-0-00-000002-6".parse()?;
    match shop.sell(&emma, 1).await {
        Ok(sale) => println!(
            "sold {} x {} for {} (sale #{}), {} left",
            sale.quantity, sale.isbn, sale.total, sale.sale_id, sale.stock_left
        ),
        Err(err) => println!("sale refused: {err}"),
    }
    match shop.sell(&emma, 10).await {
        Ok(sale) => println!("sold {sale:?}"),
        Err(err @ InventoryError::OutOfStock { .. }) => {
            println!("sale refused: {err} (nothing was changed)")
        }
        Err(err) => return Err(err.into()),
    }
    let not_stocked: Isbn = "978-1-7185-0310-6".parse()?;
    if let Err(err) = shop.sell(&not_stocked, 1).await {
        println!("sale refused: {err}");
    }

    // 5. Aggregates.
    println!("\n== the report");
    for line in shop.sales_report().await? {
        println!(
            "{:<20} {} sold, {}",
            line.title, line.copies_sold, line.revenue
        );
    }
    println!("shelf value: {}", shop.stock_value().await?);

    shop.pool().close().await;
    Ok(())
}
