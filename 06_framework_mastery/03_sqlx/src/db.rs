//! Opening a connection pool and running the migrations.

use std::str::FromStr;
use std::time::Duration;

use sqlx::SqlitePool;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

/// Every file in `migrations/`, embedded into the binary at compile time.
///
/// `sqlx::migrate!()` reads `./migrations` (next to `Cargo.toml`) while the
/// crate compiles, so the program needs no `.sql` files at run time.
pub static MIGRATOR: Migrator = sqlx::migrate!();

/// Opens a pool for a database URL such as `sqlite:bookshop.db?mode=rwc`
/// (`mode=rwc` creates the file if it's missing) or `sqlite::memory:`.
///
/// A pool keeps a few connections open and lends them out, so concurrent
/// tasks don't wait for each other or pay to reconnect every time.
pub async fn connect(url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(url)?;
    let in_memory = url.contains(":memory:") || url.contains("mode=memory");
    let mut pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3));
    if in_memory {
        // An in-memory database vanishes when its last connection closes, so
        // keep one connection open and never retire it.
        pool = pool
            .min_connections(1)
            .idle_timeout(None)
            .max_lifetime(None);
    }
    pool.connect_with(options).await
}

/// A fresh, private, migrated in-memory database: ideal for tests.
///
/// Each call to `connect("sqlite::memory:")` gets its *own* database, so
/// tests never see each other's rows, and nothing is left on disk.
pub async fn connect_in_memory() -> Result<SqlitePool, sqlx::Error> {
    let pool = connect("sqlite::memory:").await?;
    MIGRATOR.run(&pool).await?;
    Ok(pool)
}
