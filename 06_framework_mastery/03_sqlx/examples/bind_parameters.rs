//! Bind parameters vs `format!`: why gluing user input into SQL is SQL injection.
//! Run: cargo run -p sqlx_database --example bind_parameters

use sqlx::{QueryBuilder, Sqlite, SqlitePool};

async fn setup() -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    sqlx::raw_sql(
        "CREATE TABLE users (name TEXT NOT NULL, secret TEXT NOT NULL);
         INSERT INTO users VALUES ('ama', 'likes jollof'), ('kofi', 'afraid of geese');",
    )
    .execute(&pool)
    .await?;
    Ok(pool)
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = setup().await?;
    // What an attacker types into the "name" box of a login form:
    let input = "nobody' OR '1'='1";

    // DON'T: glue the input into the SQL text. SQLx 0.9 refuses a `String`
    // here (exercise 1 shows the compile error); `AssertSqlSafe` is the
    // "I have checked this myself" override. Here it is a lie, on purpose.
    let sql = format!("SELECT name, secret FROM users WHERE name = '{input}'");
    println!("the SQL the database sees:\n  {sql}");
    let leaked: Vec<(String, String)> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .fetch_all(&pool)
        .await?;
    println!("format! leaked {} secrets: {leaked:?}", leaked.len());

    // DO: a placeholder `?` in the SQL, the value sent separately with `bind`.
    // The database parses the SQL first and only then plugs the value in, so
    // the value can never become SQL: it's just a strange name.
    let found: Vec<(String, String)> =
        sqlx::query_as("SELECT name, secret FROM users WHERE name = ?")
            .bind(input)
            .fetch_all(&pool)
            .await?;
    println!("bind found {} rows: {found:?}", found.len());

    // Quotes inside a bound value are fine too: no escaping needed.
    sqlx::query("INSERT INTO users VALUES (?, ?)")
        .bind("O'Brien")
        .bind("it's a secret")
        .execute(&pool)
        .await?;
    let secret: String = sqlx::query_scalar("SELECT secret FROM users WHERE name = ?")
        .bind("O'Brien")
        .fetch_one(&pool)
        .await?;
    println!("O'Brien's secret: {secret:?}");

    // Variable-length SQL (an IN list, optional filters): QueryBuilder writes
    // one placeholder per value and binds them all.
    let wanted = ["ama", "O'Brien", "nobody' OR '1'='1"];
    let mut query = QueryBuilder::<Sqlite>::new("SELECT name FROM users WHERE name IN (");
    let mut list = query.separated(", ");
    for name in wanted {
        list.push_bind(name);
    }
    query.push(") ORDER BY name");
    println!("QueryBuilder SQL: {}", query.sql().as_str());
    let names: Vec<String> = query.build_query_scalar().fetch_all(&pool).await?;
    println!("QueryBuilder found: {names:?}");
    Ok(())
}
