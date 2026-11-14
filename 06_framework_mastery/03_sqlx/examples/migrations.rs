//! Migrations: `sqlx::migrate!()` embeds `migrations/*.sql`; `run` applies the new ones.
//! Run: cargo run -p sqlx_database --example migrations

use sqlx::SqlitePool;
use sqlx_database::db::MIGRATOR;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // What `sqlx::migrate!()` embedded at compile time.
    println!("== embedded in the binary");
    for migration in MIGRATOR.iter() {
        let checksum: String = migration
            .checksum
            .iter()
            .take(4)
            .map(|b| format!("{b:02x}"))
            .collect();
        println!(
            "{} {:<26} checksum {checksum}…",
            migration.version, migration.description
        );
    }

    let pool = SqlitePool::connect("sqlite::memory:").await?;

    // First run: creates the `_sqlx_migrations` table, then applies 1, 2, 3
    // in order, each in its own transaction.
    MIGRATOR.run(&pool).await?;
    // Second run: every version is already recorded, so nothing happens.
    // That's why it is safe to call `run` every time the program starts.
    MIGRATOR.run(&pool).await?;

    println!("\n== recorded in _sqlx_migrations");
    let rows: Vec<(i64, String, bool)> = sqlx::query_as(
        "SELECT version, description, success FROM _sqlx_migrations ORDER BY version",
    )
    .fetch_all(&pool)
    .await?;
    for (version, description, success) in rows {
        println!("{version} {description:<26} success={success}");
    }

    println!("\n== the schema they built");
    let tables: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
    )
    .fetch_all(&pool)
    .await?;
    for (table,) in tables {
        let columns: Vec<String> = sqlx::query_scalar("SELECT name FROM pragma_table_info(?)")
            .bind(&table)
            .fetch_all(&pool)
            .await?;
        println!("{table}: {}", columns.join(", "));
    }

    // A database that is AHEAD of the binary (a newer version of the program
    // migrated it) makes `run` fail instead of guessing.
    sqlx::query(
        "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) \
         VALUES (4, 'from the future', TRUE, X'00', 0)",
    )
    .execute(&pool)
    .await?;
    println!("\n== run against a database from a newer program");
    println!("{}", MIGRATOR.run(&pool).await.unwrap_err());
    Ok(())
}
