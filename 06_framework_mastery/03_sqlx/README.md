<!-- hand-written lesson: gen_course must not overwrite -->
# SQLx: async SQL databases

**Docs:** [SQLx on GitHub](https://github.com/launchbadge/sqlx) · [API docs](https://docs.rs/sqlx) · [FAQ](https://github.com/launchbadge/sqlx/blob/main/FAQ.md) · [`sqlx-cli`](https://crates.io/crates/sqlx-cli)
· SQLite: [SQL as understood by SQLite](https://www.sqlite.org/lang.html), [datatypes](https://www.sqlite.org/datatype3.html)
· Background: [OWASP, SQL injection](https://owasp.org/www-community/attacks/SQL_Injection)

Run it: `cargo run -p sqlx_database`  ·  Test it: `cargo test -p sqlx_database`

Almost every real service keeps its data in a database, and SQL is how you talk to one. [SQLx](https://docs.rs/sqlx) lets async Rust send SQL to PostgreSQL, MySQL or SQLite. It is **not an ORM**: you write real SQL, and SQLx runs it, binds your values safely, and turns the rows back into Rust types. Its party trick is that it can check your SQL against the real database **while your program compiles**.

After this lesson you will be able to:

- open a **connection pool** and run queries with `query`, `query_as` and `query_scalar`;
- pass values with **bind parameters**, and explain why gluing them into SQL with `format!` is SQL injection;
- turn rows into structs with `#[derive(FromRow)]`;
- make several changes **all or nothing** with transactions;
- version your schema with **migrations** (`sqlx::migrate!`);
- explain **compile-time checked queries** (`query!`, `query_as!`), `DATABASE_URL` and offline mode (`.sqlx/`);
- handle database errors properly, and **test** against an in-memory database.

The mini project is a **bookshop inventory**: books, authors, stock and sales, with migrations, transactions and 22 tests.

> 💡 **The big picture: a library with a librarian.** The database is a huge **library** that never forgets. You can't wander in and rearrange the shelves yourself: you hand a **request slip** (a SQL query) to the **librarian** (the database server), who fetches or files things for you. SQLx is your **messenger**: it carries slips to the librarian and brings the answers back as tidy Rust values. A **pool** is a small team of messengers who already know the way, so nobody waits at the door. **Bind parameters** are the boxes on the slip where you write the details, so nobody can sneak extra orders in. A **transaction** is a sealed envelope of slips: the librarian does all of them or none of them. **Migrations** are the numbered building plans that turned an empty room into this library, one extension at a time.

## Before you start: SQLx, SQLite and the C compiler

```toml
[dependencies]
sqlx = { version = "0.9.0", default-features = false, features = ["runtime-tokio", "sqlite", "macros", "migrate"] }
tokio = { version = "1.54.1", features = ["full"] }
```

| Feature | What it adds |
|---|---|
| `runtime-tokio` | runs on Tokio ([lesson 06/02](../02_tokio/README.md)) |
| `sqlite` (or `postgres`, `mysql`) | the database driver |
| `macros` | `query!`, `query_as!`, `query_scalar!`, `#[derive(FromRow)]`, `#[sqlx::test]` |
| `migrate` | `sqlx::migrate!()` and the `Migrator` |

This lesson uses **SQLite**: the whole database lives in one file, or only in memory, so every example and test runs without installing a database server. SQLx 0.9 requires Rust 1.94 or newer.

**SQLite needs a C compiler.** The `sqlite` feature builds SQLite itself from its C source (the `libsqlite3-sys` crate, "bundled"). On Windows that means the MSVC Build Tools, which the `x86_64-pc-windows-msvc` Rust toolchain already needs for linking, so it usually just works; on Linux, `gcc` or `clang` (`build-essential`). The `postgres` and `mysql` drivers are pure Rust, but need a server to talk to (section 10).

> 💡 **Analogy: choosing a library.** SQLite is a **notebook in your backpack**: it goes everywhere, needs no building, and is perfect for one person (or one program). PostgreSQL is the **town library**: a separate building with staff, opening hours and room for thousands of visitors at once. The messenger (SQLx) speaks to both.

## 1. Connecting: a pool of connections

```rust
use sqlx::sqlite::SqlitePoolOptions;

let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .acquire_timeout(Duration::from_secs(3))
    .connect("sqlite:bookshop.db?mode=rwc")   // or "sqlite::memory:"
    .await?;
```

- Opening a database connection is slow (a network handshake, logging in). A **pool** opens a few, keeps them, and **lends** one to each query. `&pool` is what you pass to queries; it is `Clone` (cheap: clones share the connections) and `Send + Sync`, so put it in your app state and share it between tasks.
- `SqlitePool::connect(url)` is the short form with default options.
- SQLite URLs: `sqlite:bookshop.db` (must exist), `sqlite:bookshop.db?mode=rwc` (create if missing), `sqlite::memory:` (in memory).
- **In-memory gotchas.** Each `connect("sqlite::memory:")` makes a **separate, private** database: perfect for tests. All connections of one pool share it. It **vanishes when its last connection closes**, so [`src/db.rs`](src/db.rs) keeps one open (`min_connections(1)`, `idle_timeout(None)`, `max_lifetime(None)`).
- `pool.close().await` waits for borrowed connections to come back, then closes them.

> 💡 **Analogy: a taxi rank.** Phoning for a new taxi for every trip means waiting each time. A **rank** keeps five taxis waiting outside: you take one, ride, and it comes back for the next person. `max_connections` is the size of the rank, and `acquire_timeout` is how long you'll wait when all five are out before giving up.

## 2. Running queries

```rust
// execute: run a statement, get back how many rows changed
let done = sqlx::query("INSERT INTO pets (name, legs) VALUES (?, ?)")
    .bind("Rex").bind(4)
    .execute(&pool).await?;
done.rows_affected();      // 1
done.last_insert_rowid();  // the new id (SQLite)

// fetch_one / fetch_all / fetch_optional / fetch (a stream)
let row = sqlx::query("SELECT name, legs FROM pets WHERE id = ?").bind(2).fetch_one(&pool).await?;
let name: String = row.get("name");   // by name (or row.get(0) by position)

// query_scalar: the first column, already decoded
let total: i64 = sqlx::query_scalar("SELECT SUM(legs) FROM pets").fetch_one(&pool).await?;
```

| Method | Returns | When there are no rows |
|---|---|---|
| `execute` | a result with `rows_affected()` | `Ok`, 0 rows affected |
| `fetch_one` | one row | `Err(sqlx::Error::RowNotFound)` |
| `fetch_optional` | `Option<row>` | `Ok(None)` |
| `fetch_all` | `Vec<row>` | `Ok(vec![])` |
| `fetch` | a `Stream` of rows (use `futures::TryStreamExt::try_next`) | the stream ends |

Every method takes an **executor**: `&pool`, `&mut conn` (a single connection) or `&mut *tx` (a transaction, section 5). The real output of `connect_and_query`:

```text
inserted Rex: 1 row, id 1
fetch_one      -> Polly has 2 legs
fetch_all      -> ["Rex", "Tiddles"]
fetch_optional -> found Godzilla? false
query_scalar   -> 10 legs altogether
fetch (stream) -> Rex Polly Nemo Tiddles
```

> 💡 **Analogy: asking the librarian.** `fetch_one` is "bring me **the** book with this number", and it's a problem if there isn't one. `fetch_optional` is "bring it **if** you have it". `fetch_all` is "bring me **every** book about dogs" (an armful, maybe empty). `fetch` is "bring them **one at a time**", for when the armful would be too heavy to carry. `execute` is "please file these", and the librarian tells you how many she filed.

▶ `cargo run -p sqlx_database --example connect_and_query`

## 3. Bind parameters, and why `format!` is SQL injection

Never build SQL by pasting values into it. A login form that runs:

```rust
let sql = format!("SELECT name, secret FROM users WHERE name = '{input}'");
```

with the input `nobody' OR '1'='1` sends the database `... WHERE name = 'nobody' OR '1'='1'`, which is true for **every** row. The real output of `bind_parameters`:

```text
the SQL the database sees:
  SELECT name, secret FROM users WHERE name = 'nobody' OR '1'='1'
format! leaked 2 secrets: [("ama", "likes jollof"), ("kofi", "afraid of geese")]
bind found 0 rows: []
```

The fix is a **placeholder** in the SQL and the value sent **separately**:

```rust
sqlx::query_as("SELECT name, secret FROM users WHERE name = ?")   // Postgres writes $1, $2, …
    .bind(input)
    .fetch_all(&pool).await?;
```

The database parses the SQL first and only then plugs the values in, so a value can never become SQL. Quotes need no escaping (`O'Brien` just works), and the database can reuse the prepared statement.

**SQLx 0.9 makes the safe way the easy way.** `query`, `query_as`, `query_scalar` and `raw_sql` accept `impl SqlSafeStr`, which only `&'static str` (SQL written in your code) implements. A `String` built at run time is a compile error, the real one from exercise 1:

```text
error[E0277]: dynamic SQL strings should be audited for possible injections
   --> 06_framework_mastery\03_sqlx\exercises\ex01_fix_format_sql.rs:22:24
    |
 22 |     sqlx::query_scalar(sql).fetch_optional(pool).await
    |     ------------------ ^^^ dynamic SQL string
    |     |
    |     required by a bound introduced by this call
    |
    = help: the trait `SqlSafeStr` is not implemented for `String`
    = note: prefer literal SQL strings with bind parameters or `QueryBuilder` to add dynamic data to a query.
            
            To bypass this error, manually audit for potential injection vulnerabilities and wrap with `AssertSqlSafe()`.
            For details, see the docs for `SqlSafeStr`.
            
    = note: this trait is only implemented for `&'static str`, not all `&str` like the compiler error may suggest
```

- `sqlx::AssertSqlSafe(string)` is the escape hatch: "I checked this string myself". It is for SQL built from **your own** trusted pieces (say, a column name picked from a fixed list), never for user input. The example uses it once, on purpose, to show the attack.
- Pieces of SQL known at compile time can be glued with `concat!`, which still gives a `&'static str`: [`src/inventory.rs`](src/inventory.rs) does this with a `select_books!()` macro.
- **SQL whose shape changes at run time** (optional filters, an `IN (?, ?, ?)` list, `LIMIT`) is what `QueryBuilder` is for: `push` adds your SQL text, `push_bind` adds a `?` and binds the value.

```rust
let mut query = QueryBuilder::<Sqlite>::new("SELECT name FROM users WHERE name IN (");
let mut list = query.separated(", ");
for name in wanted { list.push_bind(name); }
query.push(") ORDER BY name");
// SELECT name FROM users WHERE name IN (?, ?, ?) ORDER BY name
let names: Vec<String> = query.build_query_scalar().fetch_all(&pool).await?;
```

Bind parameters only stand in for **values**. Table names, column names and `ASC`/`DESC` can't be bound: choose them from a fixed list in your code.

> 💡 **Analogy: a form with boxes.** Writing a letter to the librarian ("please fetch the book called Emma") lets a trickster write "Emma, and also open the safe" and the librarian obeys the whole letter. A **form** has a printed question and a **box** for the title. Whatever the trickster writes in the box, it's only ever read as a title. `format!` is the letter; `?` and `bind` are the form. SQLx 0.9 won't even carry letters it didn't see you write.

▶ `cargo run -p sqlx_database --example bind_parameters`

## 4. Rows into structs: `FromRow`

```rust
#[derive(Debug, sqlx::FromRow)]
struct Pet {
    id: PetId,                // a newtype: #[derive(sqlx::Type)] #[sqlx(transparent)]
    name: String,
    #[sqlx(rename = "legs")]  // column `legs`, field `leg_count`
    leg_count: i64,
    nickname: Option<String>, // a column that may be NULL
    #[sqlx(default)]          // not in the query: Default::default()
    adopted: bool,
}

let pets: Vec<Pet> = sqlx::query_as("SELECT id, name, legs, nickname FROM pets")
    .fetch_all(&pool).await?;
```

- The derive matches columns to fields **by name**; extra columns are ignored, a missing one is a run-time error (`no column found for name: legs`).
- **Tuples** are `FromRow` by position: `query_as::<_, (String, i64)>(…)`.
- `#[derive(sqlx::Type)] #[sqlx(transparent)]` on a one-field struct makes it bind and decode exactly like the value inside. The bookshop's `Pence(i64)` and `Isbn(String)` work this way, so validated types go straight into and out of the database.
- `impl FromRow<'_, SqliteRow>` by hand when a row needs real work to become a value.
- **SQLite trap:** NULL in a non-`Option` field is **not** an error with SQLite in SQLx 0.9. It silently becomes `""`, `0` or `false` (real output: `NULL into String: ("",) (silently empty!)`). PostgreSQL refuses with "unexpected null". Either way: a nullable column needs an `Option<T>` field.
- Money is `INTEGER` pence, never `REAL`: floats can't hold most decimal fractions exactly.

> 💡 **Analogy: a sorting machine with labelled trays.** Each row arrives as a pile of loose cards (the columns). `FromRow` is a machine that drops each card into the tray **with the same label** on your struct. `rename` is a sticker that says "the card called `legs` goes in the tray called `leg_count`". An `Option` tray is allowed to stay empty; an ordinary tray must get a card.

▶ `cargo run -p sqlx_database --example from_row`

## 5. Transactions: all or nothing

```rust
let mut tx = pool.begin().await?;                      // BEGIN
sqlx::query("UPDATE jars SET coins = coins - ? WHERE owner = ?")
    .bind(4).bind("ama").execute(&mut *tx).await?;     // note: &mut *tx
sqlx::query("UPDATE jars SET coins = coins + ? WHERE owner = ?")
    .bind(4).bind("kofi").execute(&mut *tx).await?;
tx.commit().await?;                                     // COMMIT: both, or…
```

- Until `commit`, nobody else sees the changes. `tx.rollback()` undoes them, and so does **dropping** `tx` without committing, which is why `?` inside a transaction is safe: an early return rolls everything back.
- **Run queries on `&mut *tx`**, not `&mut tx`. A `Transaction` derefs to the connection inside it, and `&mut SqliteConnection` is an executor; `&mut Transaction` isn't. The real error from exercise 2:

```text
error[E0277]: the trait bound `&mut Transaction<'_, Sqlite>: Executor<'_>` is not satisfied
   --> 06_framework_mastery\03_sqlx\exercises\ex02_fix_transaction_executor.rs:17:18
    |
 17 |         .execute(&mut tx)
    |          ------- ^^^^^^^ the trait `Executor<'_>` is not implemented for `&mut Transaction<'_, Sqlite>`
    |          |
    |          required by a bound introduced by this call
```

  (A function whose parameter is spelled `conn: &mut SqliteConnection` *can* take `&mut tx`: there, deref coercion kicks in. Generic `impl Executor` parameters get no coercion.)
- **`commit` takes `self`**, so the transaction is gone afterwards. The real error from exercise 3:

```text
error[E0382]: borrow of moved value: `tx`
   --> 06_framework_mastery\03_sqlx\exercises\ex03_fix_use_after_commit.rs:27:24
    |
 17 |     let mut tx = pool.begin().await?;
    |         ------ move occurs because `tx` has type `Transaction<'_, Sqlite>`, which does not implement the `Copy` trait
...
 23 |     tx.commit().await?;
    |        -------- `tx` moved due to this method call
...
 27 |         .execute(&mut *tx)
    |                        ^^ value borrowed here after move
```

- Write helpers that take `&mut SqliteConnection`: they work on a transaction, a pooled connection (`pool.acquire()`), or a plain connection, so callers decide what is atomic (`insert_book` in [`src/inventory.rs`](src/inventory.rs)).
- `begin` on a transaction starts a **savepoint**: a part that can be rolled back on its own.
- **Check and change in one statement.** Reading the stock, deciding, then writing is a race: two tills can both read "1 left" and both sell it. The bookshop's `sell` does `UPDATE books SET stock = stock - ? WHERE isbn = ? AND stock >= ? RETURNING …` and treats "no row" as "not enough". A test fires 12 sales at 5 copies at once and checks exactly 5 succeed.

The real output of `transactions`:

```text
start                              [("ama", 10), ("kofi", 3)]
after commit (ama gave 4)          [("ama", 6), ("kofi", 7)]
after rollback (kofi changed mind) [("ama", 6), ("kofi", 7)]
moving 100 coins: error returned from database: (code: 275) CHECK constraint failed: coins >= 0
after the failed move              [("ama", 6), ("kofi", 7)]
no transaction: 2 coins vanished   [("ama", 6), ("kofi", 5)]
inside the savepoint kofi has 1; changed our mind
outer committed, inner undone      [("ama", 5), ("kofi", 6)]
```

> 💡 **Analogy: swapping stickers.** Ama gives Kofi four stickers. If she hands them over but Kofi's book falls in a puddle before he sticks them in, the stickers are lost. A transaction is a **referee**: nothing counts until both halves are done and the referee says "done!" (`commit`). If anything goes wrong, or the referee walks off (`drop`), everything goes back to how it was. A savepoint is "let's try this bit, and undo just this bit if it goes wrong".

▶ `cargo run -p sqlx_database --example transactions`

## 6. Migrations: the schema under version control

```text
migrations/
  0001_create_authors_and_books.sql
  0002_create_sales.sql
  0003_add_published_year.sql
```

```rust
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!(); // reads ./migrations at COMPILE time
MIGRATOR.run(&pool).await?;                                      // applies the new ones
```

- A migration is a SQL file named `<VERSION>_<DESCRIPTION>.sql`. The version is a number (`sqlx-cli` uses a timestamp such as `20261114120000`), and they run in order. Files ending `.up.sql` / `.down.sql` make **reversible** migrations.
- `sqlx::migrate!()` embeds every file into the binary, so the program needs no `.sql` files when it runs.
- `run` creates a `_sqlx_migrations` table, then applies, **each in its own transaction**, every migration not yet recorded there. Running it again does nothing, so it's safe at every start-up.
- Each applied migration's **checksum** (SHA-384 of the file) is stored. **Never edit a migration that has run anywhere**: `run` then fails with "migration 1 was previously applied but has been modified". Add a new migration instead (0003 here adds a column). This lesson's `.gitattributes` pins `migrations/*.sql` to LF line endings so a Windows checkout can't change the checksums.
- A database migrated by a **newer** program refuses older code (real output below), instead of guessing.
- `sqlx-cli` (`cargo install sqlx-cli`, not needed here) adds `sqlx database create`, `sqlx migrate add <name>`, `sqlx migrate run` and `sqlx migrate revert`.

The real output of `migrations` (abridged):

```text
== embedded in the binary
1 create authors and books   checksum 8935adb1…
2 create sales               checksum 1dc0f692…
3 add published year         checksum 37a9fc49…

== the schema they built
_sqlx_migrations: version, description, installed_on, success, checksum, execution_time
authors: id, name
books: id, isbn, title, author_id, price_pence, stock, published_year
sales: id, book_id, quantity, total_pence, sold_at

== run against a database from a newer program
migration 4 was previously applied but is missing in the resolved migrations
```

> 💡 **Analogy: numbered building plans.** The library started as an empty room. Plan 1 built the first shelves, plan 2 added a counter, plan 3 added a new column of drawers. A **logbook** by the door (`_sqlx_migrations`) records which plans are done, so the builders only carry out new ones. You never redraw plan 1 after it's built: you write plan 4.

▶ `cargo run -p sqlx_database --example migrations`

## 7. Compile-time checked queries: `query!` and `query_as!`

The functions above check SQL when it **runs**. The macros check it when the program **compiles**:

```rust
let row = sqlx::query!("SELECT title, stock, published_year FROM books WHERE isbn = ?", isbn)
    .fetch_one(&pool).await?;
row.title;           // String
row.published_year;  // Option<i64>: the column has no NOT NULL

let cheap = sqlx::query_as!(PriceTag, "SELECT title, price_pence FROM books WHERE price_pence <= ?", max)
    .fetch_all(&pool).await?;   // no FromRow needed: the macro writes the mapping and checks each field

let count = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!: i64" FROM books"#).fetch_one(&pool).await?;
```

To check a query, the macro needs the schema, and gets it one of two ways:

- **Online:** `DATABASE_URL` (an environment variable, or a `.env` file in the crate or a parent folder) points at a migrated database; the macro asks it to describe each query.
- **Offline:** with no `DATABASE_URL`, the macro reads saved descriptions from the crate's `.sqlx/` folder (one `query-<hash>.json` per query). `cargo sqlx prepare` (from `sqlx-cli`) writes that folder; you **commit** it, so CI and other developers can build without a database. `cargo sqlx prepare --check` in CI fails if it's out of date, and `SQLX_OFFLINE=true` forces offline mode even when `DATABASE_URL` is set.

With neither, the build fails (the real error, with `.sqlx/` moved away):

```text
error: set `DATABASE_URL` to use query macros online, or run `cargo sqlx prepare` to update the query cache
  --> 06_framework_mastery\03_sqlx\examples\compile_time_checked.rs:37:15
   |
37 |       let row = sqlx::query!(
   |  _______________^
38 | |         "SELECT title, stock, published_year FROM books WHERE isbn = ?",
39 | |         isbn
40 | |     )
   | |_____^
```

This lesson commits [`.sqlx/`](.sqlx) for its one example that uses the macros, so it builds anywhere. It was generated **without** `sqlx-cli`, by doing what `cargo sqlx prepare` does: point `DATABASE_URL` at a migrated database and `SQLX_OFFLINE_DIR` at the folder, then compile. From the workspace root, in Git Bash (both paths are relative to the root, where Cargo runs the compiler; `target/` is ignored by Git):

```bash
cargo run -p sqlx_database -- "sqlite:06_framework_mastery/03_sqlx/target/schema.db?mode=rwc"   # create + migrate
touch 06_framework_mastery/03_sqlx/examples/compile_time_checked.rs                            # force a rebuild
DATABASE_URL="sqlite:06_framework_mastery/03_sqlx/target/schema.db" \
SQLX_OFFLINE_DIR="06_framework_mastery/03_sqlx/.sqlx" \
cargo build -p sqlx_database --example compile_time_checked
```

What it catches, for real. A misspelt column, with `DATABASE_URL` set:

```text
error: error returned from database: (code: 1) no such column: titel
  --> 06_framework_mastery\03_sqlx\examples\compile_time_checked.rs:37:15
   |
37 |       let row = sqlx::query!(
   |  _______________^
38 | |         "SELECT titel, stock, published_year FROM books WHERE isbn = ?",
```

Two arguments for one `?` is `error: expected 1 parameters, got 2` (no error code). A `query_as!` struct field of the wrong type (`price_pence: String` for an `INTEGER` column) is `error[E0277]: the trait bound `String: From<i64>` is not satisfied`.

- Override what the macro infers with the column alias: `AS "name!"` (never NULL), `AS "name?"` (may be NULL), `AS "name: Type"` (decode as `Type`).
- **The trade-off:** stronger guarantees, but builds need the database or a fresh `.sqlx/`, and a query that is assembled at run time (`QueryBuilder`) can't be checked. Many projects use the macros for fixed queries and `QueryBuilder` for search screens. This lesson's library uses the runtime functions so you can read every step.

> 💡 **Analogy: a spell-checker that phones the librarian.** While you write your request slips, the checker rings the library and reads each one out: "Is there a shelf called `titel`?" "No!" and your pen stops before the slip is ever sent. **Offline mode** is the checker using **last week's notes** from those calls (`.sqlx/`), so it works on a train with no signal, as long as you refresh the notes whenever you change a slip.

▶ `cargo run -p sqlx_database --example compile_time_checked`

## 8. Error handling

`sqlx::Error` is one enum. The variants you'll meet most:

| Variant | Means | Typical response |
|---|---|---|
| `RowNotFound` | `fetch_one` found no row | "not found", or use `fetch_optional` |
| `Database(e)` | the database refused: constraint, syntax, lock… | look at `e.kind()` |
| `PoolTimedOut` | no connection free within `acquire_timeout` | "busy, try again" (HTTP 503) |
| `ColumnNotFound`, `ColumnDecode` | the row doesn't fit your type | a bug in your query or struct |
| `Io`, `Tls`, `Protocol` | the connection broke | retry, or fail loudly |

Ask a database error **what kind** it is instead of matching message text (which differs between databases). The real output of `error_handling`:

```text
fetch_one, no row:      RowNotFound
duplicate ISBN  -> UniqueViolation     code Some("2067"): UNIQUE constraint failed: books.isbn
unknown author  -> ForeignKeyViolation code Some("787"): FOREIGN KEY constraint failed
negative price  -> CheckViolation      code Some("275"): CHECK constraint failed: price_pence >= 0
missing title   -> NotNullViolation    code Some("1299"): NOT NULL constraint failed: books.title

add_book(duplicate):  a book with ISBN 9780000000019 already exists
remove_book(sold):    book 9780000000026 has been sold, so it can't be deleted
sell(5):              only 1 left
parse(bad ISBN):      invalid input: an ISBN-13 has 13 digits, found 4

second acquire:       pool timed out while waiting for an open connection (PoolTimedOut)
```

The bookshop wraps this in its own `InventoryError` ([`src/error.rs`](src/error.rs), with `thiserror` from [lesson 02/05](../../02_core_types_and_errors/05_error_handling/README.md)): `err.as_database_error().is_some_and(|db| db.is_unique_violation())` becomes `DuplicateIsbn`, a foreign-key failure on delete becomes `HasSales`, and everything else passes through as `Database(#[from] sqlx::Error)`. Callers (and the web API in the [capstone](../07_capstone_rest_api/README.md)) match on meanings, not on SQL. **Validate first, constrain last:** `Isbn::parse` and `"12.99".parse::<Pence>()` reject bad input before any SQL runs; the table's `CHECK`, `UNIQUE` and `REFERENCES` constraints are the safety net that also catches what other programs (or bugs) send.

> 💡 **Analogy: why the librarian said no.** "No" isn't enough to know what to do next. "We already have that book" (unique), "that author isn't in our records" (foreign key), "a price can't be negative" (check) and "all the messengers are busy" (pool timeout) each need a different reply to the customer. So you ask for the **reason code**, not just the frown.

▶ `cargo run -p sqlx_database --example error_handling`

## 9. Testing with an in-memory database

- **Pure logic needs no database.** `money.rs` and `isbn.rs` are plain functions with plain unit tests.
- **Repository tests use a real database**, not a mock: `db::connect_in_memory()` opens a fresh private in-memory SQLite and runs the migrations, in a few milliseconds. Every test gets its own, so they run in parallel and never see each other's rows ([`tests/inventory.rs`](tests/inventory.rs)).
- **`#[sqlx::test]`** is SQLx's own test attribute: it gives each test a fresh database with `./migrations` applied. For SQLite that's a file under `target/sqlx/test-dbs/` next to `Cargo.toml` (deleted when the test passes, and ignored by Git); for Postgres it creates and drops a whole database on the server in `DATABASE_URL` ([`tests/sqlx_test_attribute.rs`](tests/sqlx_test_attribute.rs)).
- Test the **unhappy paths**: a duplicate, an out-of-stock sale that must change nothing, hostile search text, and concurrency (12 tasks selling 5 copies).

```rust
#[sqlx::test]
async fn each_test_gets_its_own_database(pool: SqlitePool) {
    let shop = Inventory::new(pool);
    shop.import_catalogue(&sample_catalogue()).await.unwrap();
    assert_eq!(shop.count_books().await.unwrap(), 5);
}
```

> 💡 **Analogy: a toy library.** To practise being a librarian you don't use the real town library: you set up a **toy library on the kitchen table**, play, and pack it away. Each test gets its own fresh toy library, so one game can't mess up another, and it's put away in a blink.

## 10. Beyond SQLite: the same code on PostgreSQL

Switching to Postgres is mostly mechanical:

| | SQLite | PostgreSQL |
|---|---|---|
| Feature | `sqlite` (compiles C) | `postgres` (pure Rust) |
| Pool type | `SqlitePool` | `PgPool` |
| URL | `sqlite:shop.db?mode=rwc` | `postgres://user:pass@localhost:5432/shop` |
| Placeholders | `?` | `$1`, `$2`, … (reusable: `$1` twice is one value) |
| Id column | `INTEGER PRIMARY KEY` | `BIGINT GENERATED ALWAYS AS IDENTITY` |
| NULL into non-`Option` | silently `""` / `0` | error: unexpected null |
| Concurrent writers | one at a time (fine for small apps) | many |

```bash
docker run --rm -e POSTGRES_PASSWORD=change-me -p 5432:5432 postgres:17   # a throwaway local server
```

`DatabaseError::kind()`, transactions, `FromRow`, migrations and `QueryBuilder` work the same. To support **both** at run time there is `sqlx::any` (`AnyPool`), at the cost of the lowest common denominator.

> 💡 **Analogy:** moving from your backpack notebook to the town library: the request slips are almost the same, but the library numbers its boxes (`$1`, `$2`) instead of drawing blank ones (`?`), and it's strict about empty boxes.

## The mini project: a bookshop inventory

| File | What's in it |
|---|---|
| [`migrations/`](migrations) | three migrations: authors and books, sales, then a new column |
| [`src/money.rs`](src/money.rs) | `Pence(i64)`: parse `"£12.99"`, display, overflow-checked `times` |
| [`src/isbn.rs`](src/isbn.rs) | `Isbn`: ISBN-13 validation with the check digit |
| [`src/db.rs`](src/db.rs) | `connect` (pool options, in-memory care), `MIGRATOR`, `connect_in_memory` |
| [`src/models.rs`](src/models.rs) | `Book` and `SalesLine` (`FromRow`), `NewBook::parse`, `Sale`, `BookFilter` |
| [`src/inventory.rs`](src/inventory.rs) | the repository: `add_book`, `import_catalogue` (all or nothing), `find_book`, `search` (`QueryBuilder`), `restock`, `sell` (transaction + atomic update), `sales_report`, `remove_book` |
| [`src/error.rs`](src/error.rs) | `InventoryError`, and constraint violations by kind |
| [`src/main.rs`](src/main.rs) | the demo, in memory or on a file |

The real output of `cargo run -p sqlx_database`:

```text
== database: sqlite::memory:
migration 1: create authors and books
migration 2: create sales
migration 3: add published year

== imported 5 books in one transaction

== the shelf
9780000000057 Alice's Adventures in Wonderland   Lewis Carroll              £5.99 x0
9780000000026 Emma                               Jane Austen                £7.50 x2
9780000000040 Frankenstein                       Mary Shelley               £9.25 x1
9780000000019 Pride and Prejudice                Jane Austen                £8.99 x4
9780000000033 Treasure Island                    Robert Louis Stevenson     £6.99 x5
in stock and at most £8.00: ["Emma", "Treasure Island"]

== the till
sold 1 x 9780000000026 for £7.50 (sale #1), 1 left
sale refused: only 1 of 9780000000026 in stock, cannot sell 10 (nothing was changed)
sale refused: no book with ISBN 9781718503106

== the report
Emma                 1 sold, £7.50
shelf value: £87.66
```

Give it a file (`cargo run -p sqlx_database -- "sqlite:bookshop.db?mode=rwc"`) and run it twice: the second run finds the migrations applied and the books already there, sells the last copy of Emma, and the report shows `2 sold, £15.00`.

> 💡 **Analogy:** a real little shop: a **catalogue** (books and authors), a **till** that takes the book off the shelf and writes the receipt in one go, a **rule book** at the door (constraints), and a **logbook** of how the shop was built (migrations).

## Common mistakes

| You wrote | What happens | Usually the fix |
|---|---|---|
| `sqlx::query(format!("… '{input}'"))` | E0277 dynamic SQL strings should be audited for possible injections | A `?` placeholder and `.bind(input)` |
| `sqlx::AssertSqlSafe(format!(…user input…))` | compiles, and is SQL injection | Only for SQL built from your own trusted pieces |
| `.bind()` for a table or column name | SQL error: binds are values only | Pick the name from a fixed list in code |
| `.execute(&mut tx)` | E0277 `&mut Transaction<'_, Sqlite>: Executor<'_>` is not satisfied | `.execute(&mut *tx)` |
| A query on `tx` after `tx.commit()` | E0382 borrow of moved value: `tx` | Everything inside the transaction comes before `commit` |
| Several related writes on `&pool` | a failure half way leaves half the change | One transaction |
| Read the stock, then write the new value | two requests sell the last copy twice | One conditional `UPDATE … WHERE stock >= ?`, check the rows changed |
| `fetch_one` for "maybe there's a row" | `Err(RowNotFound)` | `fetch_optional` |
| A `String` field for a nullable column | SQLite: silently `""`; Postgres: unexpected null | `Option<String>` |
| `price REAL` / `f64` money | `0.1 + 0.2 != 0.3`, pennies go missing | Integer pence (`Pence(i64)`) |
| A `FromRow` struct field the query doesn't return | no column found for name | Select it, `rename`, or `#[sqlx(default)]` |
| Edited a migration that already ran | migration N was previously applied but has been modified | Add a new migration |
| Expected one in-memory DB from two `connect("sqlite::memory:")` | two separate, empty databases | Share one pool (clone it) |
| An in-memory pool left idle | the data vanishes with the last connection | `min_connections(1)`, `idle_timeout(None)`, `max_lifetime(None)` |
| `query!` with no `DATABASE_URL` and no `.sqlx/` | set `DATABASE_URL` to use query macros online, or run `cargo sqlx prepare` | Set `DATABASE_URL`, or commit `.sqlx/` |
| Changed a `query!`, or the schema, but not `.sqlx/` | a changed query: no cached data, the build fails; a changed schema: the old answers are used without a word | Re-run `cargo sqlx prepare`; `cargo sqlx prepare --check` in CI |
| `?` placeholders on Postgres | the server rejects the SQL | `$1`, `$2`, … |
| The `sqlite` feature on a machine with no C compiler | `libsqlite3-sys` fails to build | Install the C toolchain, or use Postgres |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p sqlx_database --example connect_and_query
cargo run -p sqlx_database --example bind_parameters
cargo run -p sqlx_database --example from_row
cargo run -p sqlx_database --example transactions
cargo run -p sqlx_database --example migrations
cargo run -p sqlx_database --example compile_time_checked
cargo run -p sqlx_database --example error_handling
```

**2. Do the exercises** in `exercises/`. Each one makes its own in-memory database, so there is nothing to set up:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_format_sql` | Make it compile (E0277): bind, don't `format!` | `cargo run -p sqlx_database --features exercises --bin ex01_fix_format_sql` |
| 2 | `ex02_fix_transaction_executor` | Make it compile (E0277): `&mut *tx` | `cargo run -p sqlx_database --features exercises --bin ex02_fix_transaction_executor` |
| 3 | `ex03_fix_use_after_commit` | Make it compile (E0382): commit last | `cargo run -p sqlx_database --features exercises --bin ex03_fix_use_after_commit` |
| 4 | `ex04_crud_queries` | Insert, find, filter, count, update; pass 4 tests | `cargo test -p sqlx_database --features exercises --bin ex04_crud_queries` |
| 5 | `ex05_stock_transfer` | A transaction with an atomic check, an upsert and rollback; pass 4 tests | `cargo test -p sqlx_database --features exercises --bin ex05_stock_transfer` |
| 6 | `ex06_dynamic_search` | `QueryBuilder`: optional filters, `IN` list, pages; pass 4 tests | `cargo test -p sqlx_database --features exercises --bin ex06_dynamic_search` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p sqlx_database --features solutions --bins`).

**4. The mini project:** read `src/lib.rs` from the top, run the demo on a file twice, then try:

- Add migration `0004_add_publisher.sql` (a `publisher TEXT` column), add the field to `Book` and the `select_books!` macro, and watch the tests tell you what else to update.
- Add `Inventory::price_rise(percent)`, which raises every price in **one** `UPDATE` and returns the number of books changed.
- Write a test that `sell` refuses a quantity whose total would overflow.

```bash
cargo test -p sqlx_database   # 7 unit + 13 inventory + 2 #[sqlx::test] tests, and 1 doc test
```

## Checklist

- [ ] A pool: `SqlitePoolOptions`, connection URLs, in-memory databases and their two gotchas
- [ ] `query`, `query_as`, `query_scalar`; `execute`, `fetch_one`, `fetch_optional`, `fetch_all`, `fetch`
- [ ] Bind parameters; why `format!` is SQL injection; `SqlSafeStr`, `AssertSqlSafe`, `QueryBuilder`
- [ ] `#[derive(FromRow)]`, `rename`, `default`, `Option` for NULL, `#[sqlx(transparent)]` newtypes, tuples
- [ ] Transactions: `begin`, `&mut *tx`, `commit`, `rollback`, rollback on drop, savepoints, atomic conditional updates
- [ ] Migrations: `migrations/`, `sqlx::migrate!()`, `_sqlx_migrations`, checksums, never edit an applied one
- [ ] Compile-time checked queries: `query!`, `query_as!`, `query_scalar!`, `DATABASE_URL`, `.sqlx/`, `cargo sqlx prepare`
- [ ] Error handling: `RowNotFound`, `DatabaseError::kind()`, `PoolTimedOut`, a domain error enum
- [ ] Testing: pure logic without a database, a fresh in-memory database per test, `#[sqlx::test]`
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- SQLx runs **your SQL**, asynchronously, through a **pool** of connections; `&pool` is the executor you pass around.
- `query` / `query_as` / `query_scalar` with `fetch_one` / `fetch_optional` / `fetch_all` / `fetch` / `execute` cover almost everything.
- **Values always go in with `bind`.** SQLx 0.9 refuses SQL strings built at run time; `QueryBuilder` builds variable SQL safely.
- `#[derive(FromRow)]` maps columns to fields by name; nullable columns are `Option`; newtypes with `#[sqlx(transparent)]` keep validated types all the way to the database.
- A transaction makes several changes **all or nothing**: `begin`, run on `&mut *tx`, `commit` last; dropping it rolls back.
- Migrations version the schema; `sqlx::migrate!()` embeds them and `run` applies only the new ones.
- `query!` and friends check SQL at compile time, online with `DATABASE_URL` or offline with a committed `.sqlx/`.
- Map database errors by **kind** into your own error type, and test against a fresh in-memory database.

**In one sentence for a child:** a database is a giant library that never forgets, and SQLx is a careful messenger who carries your request forms to the librarian, never lets anyone scribble extra orders in the boxes, and makes sure a swap either happens completely or not at all.

**Next:** [04 Clap](../04_clap/README.md): command-line programs with arguments, subcommands and configuration.
