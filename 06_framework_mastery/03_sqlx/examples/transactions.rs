//! Transactions: all or nothing. Commit, roll back, and roll back by dropping.
//! Run: cargo run -p sqlx_database --example transactions

use sqlx::{Connection, SqliteConnection, SqlitePool};

/// Moves pocket money between two jars. Works on any connection, including a
/// transaction: `&mut tx` coerces to `&mut SqliteConnection` here, because the
/// parameter's type is spelled out (a generic `impl Executor` needs `&mut *tx`).
async fn move_coins(
    conn: &mut SqliteConnection,
    from: &str,
    to: &str,
    coins: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE jars SET coins = coins - ? WHERE owner = ?")
        .bind(coins)
        .bind(from)
        .execute(&mut *conn)
        .await?;
    // If this one fails (a CHECK constraint, a crash, a lost connection…),
    // the first UPDATE must not count.
    sqlx::query("UPDATE jars SET coins = coins + ? WHERE owner = ?")
        .bind(coins)
        .bind(to)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

async fn show(pool: &SqlitePool, when: &str) -> Result<(), sqlx::Error> {
    let jars: Vec<(String, i64)> = sqlx::query_as("SELECT owner, coins FROM jars ORDER BY owner")
        .fetch_all(pool)
        .await?;
    println!("{when:<34} {jars:?}");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    sqlx::raw_sql(
        "CREATE TABLE jars (owner TEXT PRIMARY KEY, coins INTEGER NOT NULL CHECK (coins >= 0));
         INSERT INTO jars VALUES ('ama', 10), ('kofi', 3);",
    )
    .execute(&pool)
    .await?;
    show(&pool, "start").await?;

    // 1. begin → work → commit: both changes become visible together.
    let mut tx = pool.begin().await?;
    move_coins(&mut tx, "ama", "kofi", 4).await?;
    tx.commit().await?;
    show(&pool, "after commit (ama gave 4)").await?;

    // 2. An explicit rollback undoes everything since `begin`.
    let mut tx = pool.begin().await?;
    move_coins(&mut tx, "kofi", "ama", 7).await?;
    tx.rollback().await?;
    show(&pool, "after rollback (kofi changed mind)").await?;

    // 3. An error half way: kofi can't give 100, the CHECK (coins >= 0) fails...
    let mut tx = pool.begin().await?;
    let result = move_coins(&mut tx, "kofi", "ama", 100).await;
    println!("moving 100 coins: {}", result.unwrap_err());
    drop(tx); // ...and dropping a transaction without `commit` rolls it back.
    show(&pool, "after the failed move").await?;

    // 4. Without a transaction there is no undo. Coins sent to a jar that
    //    doesn't exist: the first UPDATE takes 2 from kofi, the second changes
    //    0 rows (not an error!), and 2 coins vanish. `acquire` lends one plain
    //    connection from the pool.
    let mut conn = pool.acquire().await?;
    move_coins(&mut conn, "kofi", "nobody", 2).await?;
    drop(conn); // back to the pool
    show(&pool, "no transaction: 2 coins vanished").await?;

    // 5. `begin` on a transaction starts a nested one, which is a SAVEPOINT:
    //    the inner part can be undone while the outer transaction carries on.
    let mut outer = pool.begin().await?;
    move_coins(&mut outer, "ama", "kofi", 1).await?; // keep this one
    {
        let mut inner = outer.begin().await?; // SAVEPOINT
        move_coins(&mut inner, "kofi", "ama", 5).await?;
        let kofi: i64 = sqlx::query_scalar("SELECT coins FROM jars WHERE owner = 'kofi'")
            .fetch_one(&mut *inner)
            .await?;
        println!("inside the savepoint kofi has {kofi}; changed our mind");
    } // `inner` dropped without commit: ROLLBACK TO SAVEPOINT
    outer.commit().await?;
    show(&pool, "outer committed, inner undone").await?;
    Ok(())
}
