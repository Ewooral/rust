//! Rows into structs: `#[derive(FromRow)]`, renames, NULL as Option, newtypes, tuples.
//! Run: cargo run -p sqlx_database --example from_row

use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row, SqlitePool};

/// A newtype that binds and decodes as the `i64` inside it.
#[derive(Debug, Clone, Copy, PartialEq, sqlx::Type)]
#[sqlx(transparent)]
struct PetId(i64);

/// The derive matches columns to fields **by name**.
#[derive(Debug, FromRow)]
struct Pet {
    id: PetId,
    name: String,
    /// The column is `legs`; the field has a better name.
    #[sqlx(rename = "legs")]
    leg_count: i64,
    /// `nickname` may be NULL, so the field must be an `Option`.
    nickname: Option<String>,
    /// Not in the table at all: `default` fills it with `Default::default()`.
    #[sqlx(default)]
    adopted: bool,
}

/// Hand-written `FromRow`, for when a row needs real work to become a value.
#[derive(Debug)]
struct Label(String);

impl FromRow<'_, SqliteRow> for Label {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        let name: String = row.try_get("name")?;
        let legs: i64 = row.try_get("legs")?;
        Ok(Label(format!("{name} ({legs} legs)")))
    }
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    sqlx::raw_sql(
        "CREATE TABLE pets (id INTEGER PRIMARY KEY, name TEXT NOT NULL,
                            legs INTEGER NOT NULL, nickname TEXT);
         INSERT INTO pets (name, legs, nickname) VALUES
             ('Rex', 4, 'Rexy'), ('Polly', 2, NULL), ('Nemo', 0, 'Fishface');",
    )
    .execute(&pool)
    .await?;

    // query_as + FromRow: one struct per row.
    let pets: Vec<Pet> = sqlx::query_as("SELECT id, name, legs, nickname FROM pets ORDER BY id")
        .fetch_all(&pool)
        .await?;
    for pet in &pets {
        println!("{pet:?}");
    }
    let legs: i64 = pets.iter().map(|p| p.leg_count).sum();
    let adopted = pets.iter().filter(|p| p.adopted).count();
    println!("{legs} legs, {adopted} adopted, first id {:?}", pets[0].id);

    // A newtype binds like its inner value.
    let polly: Pet = sqlx::query_as("SELECT * FROM pets WHERE id = ?")
        .bind(PetId(2))
        .fetch_one(&pool)
        .await?;
    println!(
        "by PetId(2): {} (nickname: {:?})",
        polly.name, polly.nickname
    );

    // Tuples implement FromRow too: column 0 → .0, column 1 → .1, …
    let pairs: Vec<(String, i64)> = sqlx::query_as("SELECT name, legs FROM pets ORDER BY legs")
        .fetch_all(&pool)
        .await?;
    println!("tuples: {pairs:?}");

    // The hand-written impl.
    let labels: Vec<Label> = sqlx::query_as("SELECT name, legs FROM pets")
        .fetch_all(&pool)
        .await?;
    let labels: Vec<String> = labels.into_iter().map(|Label(text)| text).collect();
    println!("labels: {labels:?}");

    // A missing column is a run-time error with `query_as` (the `query_as!`
    // macro would catch it at compile time instead).
    let wrong = sqlx::query_as::<_, Pet>("SELECT id, name FROM pets")
        .fetch_all(&pool)
        .await;
    println!("missing column: {}", wrong.unwrap_err());

    // So is a column whose type doesn't match the field.
    let mismatch = sqlx::query_as::<_, (i64,)>("SELECT name FROM pets")
        .fetch_all(&pool)
        .await;
    println!("TEXT into i64: {}", mismatch.unwrap_err());

    // A trap: with SQLite, NULL in a non-Option field is NOT an error. It is
    // quietly decoded as "" (or 0, or false). Postgres refuses ("unexpected
    // null"). Either way: nullable column => Option<T> field.
    let null: (String,) = sqlx::query_as("SELECT nickname FROM pets WHERE name = 'Polly'")
        .fetch_one(&pool)
        .await?;
    println!("NULL into String: {null:?} (silently empty!)");
    Ok(())
}
