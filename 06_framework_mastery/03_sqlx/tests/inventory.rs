//! End-to-end tests of the inventory against a real (in-memory) SQLite database.
//!
//! Every test calls `shop()`, which opens a brand-new private database and runs
//! the migrations, so tests can run in parallel without seeing each other's rows.

use sqlx_database::{
    BookFilter, Inventory, InventoryError, Isbn, NewBook, Pence, db, sample_catalogue,
};

async fn shop() -> Inventory {
    Inventory::new(db::connect_in_memory().await.expect("in-memory database"))
}

async fn stocked_shop() -> Inventory {
    let shop = shop().await;
    shop.import_catalogue(&sample_catalogue()).await.unwrap();
    shop
}

fn isbn(text: &str) -> Isbn {
    text.parse().unwrap()
}

const EMMA: &str = "9780000000026";
const TREASURE_ISLAND: &str = "9780000000033";

#[tokio::test]
async fn migrations_build_the_schema_and_can_run_again() {
    let shop = shop().await;
    // `connect_in_memory` already ran them once; a second run is a no-op.
    db::MIGRATOR.run(shop.pool()).await.unwrap();
    let versions: Vec<i64> =
        sqlx::query_scalar("SELECT version FROM _sqlx_migrations ORDER BY version")
            .fetch_all(shop.pool())
            .await
            .unwrap();
    assert_eq!(versions, [1, 2, 3]);
    assert_eq!(shop.count_books().await.unwrap(), 0);
}

#[tokio::test]
async fn adds_and_finds_a_book() {
    let shop = shop().await;
    let new = NewBook::parse(
        "978-1-7185-0310-6",
        "The Rust Programming Language",
        "Steve Klabnik",
        "39.99",
        3,
    )
    .unwrap();
    let book = shop.add_book(&new).await.unwrap();
    assert_eq!(book.title, "The Rust Programming Language");
    assert_eq!(book.author, "Steve Klabnik");
    assert_eq!(book.price, Pence(3999));
    assert_eq!(book.published_year, None);
    assert_eq!(shop.find_book(&new.isbn).await.unwrap(), Some(book));
    assert_eq!(shop.find_book(&isbn(EMMA)).await.unwrap(), None);
}

#[tokio::test]
async fn a_duplicate_isbn_is_refused() {
    let shop = stocked_shop().await;
    let again = NewBook::parse(EMMA, "Emma (again)", "Jane Austen", "1.00", 1).unwrap();
    let err = shop.add_book(&again).await.unwrap_err();
    assert!(
        matches!(err, InventoryError::DuplicateIsbn(ref i) if i == EMMA),
        "{err:?}"
    );
    assert_eq!(shop.count_books().await.unwrap(), 5);
}

#[tokio::test]
async fn an_import_is_all_or_nothing() {
    let shop = shop().await;
    let mut catalogue = sample_catalogue();
    catalogue.push(catalogue[0].clone()); // the last book is a duplicate
    let err = shop.import_catalogue(&catalogue).await.unwrap_err();
    assert!(matches!(err, InventoryError::DuplicateIsbn(_)), "{err:?}");
    // The first five inserts were rolled back with the sixth.
    assert_eq!(shop.count_books().await.unwrap(), 0);
    let authors: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM authors")
        .fetch_one(shop.pool())
        .await
        .unwrap();
    assert_eq!(authors, 0);
}

#[tokio::test]
async fn a_sale_takes_stock_and_records_the_sale() {
    let shop = stocked_shop().await;
    let sale = shop.sell(&isbn(TREASURE_ISLAND), 2).await.unwrap();
    assert_eq!(sale.quantity, 2);
    assert_eq!(sale.total, Pence(1398));
    assert_eq!(sale.stock_left, 3);
    assert_eq!(shop.book(&isbn(TREASURE_ISLAND)).await.unwrap().stock, 3);
    let report = shop.sales_report().await.unwrap();
    assert_eq!(report.len(), 1);
    assert_eq!(report[0].title, "Treasure Island");
    assert_eq!(report[0].copies_sold, 2);
    assert_eq!(report[0].revenue, Pence(1398));
}

#[tokio::test]
async fn selling_too_many_changes_nothing() {
    let shop = stocked_shop().await;
    let err = shop.sell(&isbn(EMMA), 3).await.unwrap_err();
    match err {
        InventoryError::OutOfStock {
            wanted, available, ..
        } => assert_eq!((wanted, available), (3, 2)),
        other => panic!("expected OutOfStock, got {other:?}"),
    }
    assert_eq!(shop.book(&isbn(EMMA)).await.unwrap().stock, 2);
    assert!(shop.sales_report().await.unwrap().is_empty());
}

#[tokio::test]
async fn bad_sales_are_refused() {
    let shop = stocked_shop().await;
    let unknown = shop.sell(&isbn("9781718503106"), 1).await.unwrap_err();
    assert!(
        matches!(unknown, InventoryError::BookNotFound(_)),
        "{unknown:?}"
    );
    let zero = shop.sell(&isbn(EMMA), 0).await.unwrap_err();
    assert!(matches!(zero, InventoryError::Invalid(_)), "{zero:?}");
}

#[tokio::test]
async fn concurrent_sales_never_sell_more_than_the_shelf_holds() {
    let shop = stocked_shop().await; // Treasure Island: 5 copies
    let mut tills = tokio::task::JoinSet::new();
    for _ in 0..12 {
        let shop = shop.clone(); // clones share the same pool
        tills.spawn(async move { shop.sell(&isbn(TREASURE_ISLAND), 1).await });
    }
    let (mut sold, mut refused) = (0, 0);
    while let Some(result) = tills.join_next().await {
        match result.unwrap() {
            Ok(_) => sold += 1,
            Err(InventoryError::OutOfStock { .. }) => refused += 1,
            Err(other) => panic!("unexpected error: {other}"),
        }
    }
    assert_eq!((sold, refused), (5, 7));
    assert_eq!(shop.book(&isbn(TREASURE_ISLAND)).await.unwrap().stock, 0);
}

#[tokio::test]
async fn restocking() {
    let shop = stocked_shop().await;
    assert_eq!(shop.restock(&isbn(EMMA), 10).await.unwrap(), 12);
    assert!(matches!(
        shop.restock(&isbn("9781718503106"), 1).await,
        Err(InventoryError::BookNotFound(_))
    ));
    assert!(matches!(
        shop.restock(&isbn(EMMA), 0).await,
        Err(InventoryError::Invalid(_))
    ));
}

#[tokio::test]
async fn search_combines_filters_and_pages() {
    let shop = stocked_shop().await;
    let titles = |books: Vec<sqlx_database::Book>| -> Vec<String> {
        books.into_iter().map(|b| b.title).collect()
    };

    let austen = BookFilter {
        author: Some("Jane Austen".into()),
        ..BookFilter::default()
    };
    assert_eq!(
        titles(shop.search(&austen).await.unwrap()),
        ["Emma", "Pride and Prejudice"]
    );

    let cheap_in_stock = BookFilter {
        max_price: Some(Pence(700)),
        in_stock_only: true,
        ..BookFilter::default()
    };
    assert_eq!(
        titles(shop.search(&cheap_in_stock).await.unwrap()),
        ["Treasure Island"]
    );

    let by_title = BookFilter {
        title_contains: Some("and".into()),
        ..BookFilter::default()
    };
    assert_eq!(
        titles(shop.search(&by_title).await.unwrap()),
        [
            "Alice's Adventures in Wonderland",
            "Pride and Prejudice",
            "Treasure Island"
        ]
    );

    let page_two = BookFilter {
        limit: Some(2),
        offset: 2,
        ..BookFilter::default()
    };
    assert_eq!(
        titles(shop.search(&page_two).await.unwrap()),
        ["Frankenstein", "Pride and Prejudice"]
    );
}

#[tokio::test]
async fn hostile_search_text_is_only_a_value() {
    let shop = stocked_shop().await;
    let attack = BookFilter {
        title_contains: Some("' OR '1'='1".into()),
        author: Some("x' OR 1=1 --".into()),
        ..BookFilter::default()
    };
    assert!(shop.search(&attack).await.unwrap().is_empty());
    assert_eq!(shop.count_books().await.unwrap(), 5);
}

#[tokio::test]
async fn removing_books() {
    let shop = stocked_shop().await;
    shop.remove_book(&isbn(TREASURE_ISLAND)).await.unwrap();
    assert_eq!(shop.count_books().await.unwrap(), 4);

    shop.sell(&isbn(EMMA), 1).await.unwrap();
    let sold = shop.remove_book(&isbn(EMMA)).await.unwrap_err();
    assert!(matches!(sold, InventoryError::HasSales(_)), "{sold:?}");

    let gone = shop.remove_book(&isbn(TREASURE_ISLAND)).await.unwrap_err();
    assert!(matches!(gone, InventoryError::BookNotFound(_)), "{gone:?}");
}

#[tokio::test]
async fn shelf_value_is_price_times_stock() {
    assert_eq!(shop().await.stock_value().await.unwrap(), Pence(0));
    let shop = stocked_shop().await;
    // 4×8.99 + 2×7.50 + 5×6.99 + 1×9.25 + 0×5.99
    assert_eq!(shop.stock_value().await.unwrap(), Pence(9516));
}
