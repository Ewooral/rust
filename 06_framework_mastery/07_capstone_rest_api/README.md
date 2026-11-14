<!-- hand-written lesson: gen_course must not overwrite -->
# Capstone: a production-style REST API

**Docs:** [axum](https://docs.rs/axum) · [SQLx](https://docs.rs/sqlx) · [Tokio](https://tokio.rs) · [tower-http](https://docs.rs/tower-http) · [tracing](https://docs.rs/tracing) · [argon2](https://docs.rs/argon2) · [jsonwebtoken](https://docs.rs/jsonwebtoken)
· Security: [OWASP Password Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html), [RFC 7519 (JWT)](https://www.rfc-editor.org/rfc/rfc7519), [OWASP REST Security Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/REST_Security_Cheat_Sheet.html)
· HTTP: [RFC 9110 (semantics, status codes)](https://www.rfc-editor.org/rfc/rfc9110) · Containers: [Docker multi-stage builds](https://docs.docker.com/build/building/multi-stage/), [Compose file reference](https://docs.docker.com/reference/compose-file/)

Run it: `APP_CONFIG_FILE=06_framework_mastery/07_capstone_rest_api/.env.example cargo run -p capstone_rest_api`, then open <http://127.0.0.1:8080/health>  ·  Test it: `cargo test -p capstone_rest_api`

This is the last lesson of the course. It puts everything together into one program shaped like a real service: a **book-review API** where people register, log in, and post reviews that only they (or an admin) may change. It uses [Axum](../01_axum/README.md) for HTTP, [Tokio](../02_tokio/README.md) underneath, and [SQLx with SQLite](../03_sqlx/README.md) for storage, and adds what a toy server leaves out: configuration, password hashing, tokens, authorisation, validation, one error format, pagination, structured logs, tests at every level, and a container image.

After this lesson you will be able to:

- organise a service in **layers** (routes → handlers → services → repositories) and say what belongs where;
- load **configuration** from a file and the environment, check it once, and keep secrets out of logs;
- store passwords safely with **Argon2id**, and authenticate requests with **JWT** bearer tokens;
- write an **extractor** that authenticates, and a **policy** that authorises;
- give every error, from every layer, **one JSON shape** with the right status code;
- **paginate** list endpoints, and **log** with `tracing` (spans, fields, request ids, JSON);
- test handlers without a network (`oneshot`), the server over TCP on port 0, and the **real binary**;
- package it with a multi-stage **Dockerfile** and **docker-compose**, and walk through a **production checklist**.

> 💡 **The big picture: a well-run library.** The **signposts** in the hall (routes) send each visitor to the right desk. The **desk clerk** (a handler) takes your form, checks it's filled in, and passes it to the back office. The **back office** (services) knows the rules: who may borrow what, how many at once. Only the **archivists** (repositories) go into the stacks (the database). At the door, a **card reader** (the `AuthUser` extractor) checks your **library card** (a token), which you got by showing your **PIN** once (your password) at registration. Your PIN is never written down anywhere, only a scrambled fingerprint of it (the Argon2 hash). And everything that happens is written in the **day book** (structured logs), each visitor with their own ticket number (request id).

## The API

| Request | Auth | Answer |
|---|---|---|
| `GET /health` | - | `200 {"status":"ok","database":"ok","version":"0.1.0"}`, or 503 |
| `POST /api/v1/auth/register` `{"email","password","display_name"}` | - | `201` the user (never the hash); `422` invalid; `409` e-mail taken |
| `POST /api/v1/auth/login` `{"email","password"}` | - | `200 {"access_token","token_type":"Bearer","expires_in":3600}`; `401` |
| `GET /api/v1/me` | token | `200` your account |
| `GET /api/v1/reviews?page=&per_page=&book=&min_rating=` | - | `200` a page: `{"items","page","per_page","total","total_pages"}` |
| `POST /api/v1/reviews` `{"book_title","rating","body"}` | token | `201` + `Location` + the review; `422` invalid |
| `GET /api/v1/reviews/{id}` | - | `200`, or `404` |
| `PATCH /api/v1/reviews/{id}` (any of the three fields) | author or admin | `200`; `403`; `404`; `422` |
| `DELETE /api/v1/reviews/{id}` | author or admin | `204`; `403`; `404` |
| anything else | - | `404`, or `405` for a wrong method, in the same JSON format |

A real session (`curl -i`, abridged):

```text
$ curl -i -X POST http://127.0.0.1:8080/api/v1/auth/register -H 'content-type: application/json' \
       -d '{"email":"ama@example.com","password":"correct horse battery","display_name":"Ama"}'
HTTP/1.1 201 Created
content-type: application/json
x-request-id: 2

{"id":1,"email":"ama@example.com","display_name":"Ama","role":"member","created_at":"2026-11-14T15:18:47Z"}

$ curl -s -X POST http://127.0.0.1:8080/api/v1/auth/login -H 'content-type: application/json' \
       -d '{"email":"ama@example.com","password":"correct horse battery"}'
{"access_token":"eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIiwicm9sZSI6Im1lbWJlciIsImlzcyI6ImNhcHN0b25lLXJlc3QtYXBpIiwiaWF0IjoxNzk0NjY5NTI4LCJleHAiOjE3OTQ2NzMxMjh9.3mQ4jZGfTC0ocl1dKs7YOeG1Hv809bibDTfTfNGWYzQ","token_type":"Bearer","expires_in":3600}

$ curl -i -X POST http://127.0.0.1:8080/api/v1/reviews -H "authorization: Bearer $TOKEN" \
       -H 'content-type: application/json' -d '{"book_title":"Treasure Island","rating":5,"body":"Pirates!"}'
HTTP/1.1 201 Created
location: /api/v1/reviews/1
x-request-id: 4

{"id":1,"author_id":1,"author_name":"Ama","book_title":"Treasure Island","rating":5,"body":"Pirates!","created_at":"2026-11-14T15:18:49Z","updated_at":"2026-11-14T15:18:49Z"}

$ curl -i -X DELETE http://127.0.0.1:8080/api/v1/reviews/1
HTTP/1.1 401 Unauthorized
www-authenticate: Bearer

{"error":{"code":"unauthorized","message":"missing Authorization header"}}

$ curl -i -X POST http://127.0.0.1:8080/api/v1/reviews -H "authorization: Bearer $TOKEN" \
       -H 'content-type: application/json' -d '{"book_title":"","rating":9}'
HTTP/1.1 422 Unprocessable Entity

{"error":{"code":"validation_failed","details":[{"field":"book_title","message":"must be 1 to 200 characters"},{"field":"rating","message":"must be from 1 to 5"}],"message":"the request has invalid fields"}}
```

▶ `cargo run -p capstone_rest_api --example api_tour` plays a longer session (two users and an admin) against a server on a free port, then stops it.

## 1. Layers: where each piece of code lives

```text
 HTTP request
     │
 routes.rs        which handler? middleware: request id, tracing, timeout, body limit
     │
 handlers/        HTTP in, HTTP out: extract, call ONE service method, build the response
     │
 service/         the rules: validate, authorise (policy.rs), hash passwords, issue tokens
     │
 repository/      the only code with SQL: domain types in, rows out
     │
 SQLite           schema from migrations/
```

| Layer | Knows about | Must not know about | Returns |
|---|---|---|---|
| routes | URLs, middleware | business rules | a `Router` |
| handlers | HTTP: extractors, status codes, headers | SQL | `Result<impl IntoResponse, AppError>` |
| services | rules, roles, validation, hashing | HTTP, JSON, SQL text | `Result<T, AppError>` |
| repositories | SQL, rows | who is asking, HTTP | `Result<T, sqlx::Error>` |

Calls only go **down**, so each layer can be read and tested on its own: [`src/domain.rs`](src/domain.rs) and [`src/service/policy.rs`](src/service/policy.rs) are pure functions with unit tests; the repositories run against an in-memory database; the handlers are tested through the router. [`AppState::build`](src/state.rs) wires them at start-up: pool → repositories → services → state.

```rust
// handlers/reviews.rs: the whole handler. No SQL, no rules.
pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,                         // authentication: the extractor
    AppPath(id): AppPath<i64>,
    AppJson(patch): AppJson<ReviewPatch>,
) -> Result<AppJson<Review>, AppError> {
    Ok(AppJson(state.reviews.update(&user, id, patch).await?))
}

// service/reviews.rs: the rules.
pub async fn update(&self, actor: &AuthUser, id: i64, patch: ReviewPatch) -> Result<Review, AppError> {
    let author = self.author(id).await?;               // 404 if missing
    authorize(actor, Action::Edit, author)?;           // 403 if not yours
    let patch = patch.validate().map_err(AppError::Validation)?;   // 422 if invalid
    self.reviews.update(id, &patch).await?.ok_or_else(|| AppError::NotFound(format!("review {id}")))
}
```

The real output of `layers_in_action`, which calls each layer directly with the same bad input:

```text
== 1. repository: SQL in, rows out. No rules at all.
inserted review 1 by Ama
rating 9 straight into SQL: error returned from database: (code: 275) CHECK constraint failed: rating BETWEEN 1 AND 5

== 2. service: validation and authorisation, in AppError terms.
rating 9 via the service:   Validation([FieldError { field: "rating", message: "must be from 1 to 5" }])
a stranger deletes it:      Forbidden

== 3. handler + router: HTTP status and JSON body.
POST /api/v1/reviews -> 422 Unprocessable Entity
{"error":{"code":"validation_failed","details":[{"field":"rating","message":"must be from 1 to 5"}],"message":"the request has invalid fields"}}
```

The services here use the concrete repositories, and tests use a real in-memory database instead of mocks. A bigger system might put a **trait** between service and repository (`trait ReviewStore`) so it can swap databases or test with fakes; that's a trade of flexibility for indirection, worth making when you need it.

> 💡 **Analogy: the library's departments.** The clerk at the desk never goes into the stacks, and the archivist never argues with visitors. When the rules change ("members may now borrow six books"), only the back office changes its rulebook. When the stacks move to a new building (a new database), only the archivists learn the new layout. Everyone else carries on as before.

▶ `cargo run -p capstone_rest_api --example layers_in_action`

## 2. Configuration: defaults, a file, the environment

Every setting comes from **defaults < a `KEY=VALUE` file (`APP_CONFIG_FILE`) < environment variables**, and is checked **once, at start-up**: a typo stops the server immediately with a clear message, instead of failing on the first request that needs it.

| Variable | Default | Notes |
|---|---|---|
| `APP_HOST`, `APP_PORT` | `127.0.0.1`, `8080` | `0.0.0.0` in a container; port `0` = any free port |
| `DATABASE_URL` | required | `sqlite::memory:`, `sqlite:capstone.db?mode=rwc` |
| `JWT_SECRET` | required | at least 32 bytes; never logged |
| `JWT_ISSUER`, `TOKEN_TTL_SECONDS` | `capstone-rest-api`, `3600` | |
| `PASSWORD_MEMORY_KIB`, `PASSWORD_ITERATIONS` | `19456`, `2` | Argon2 cost |
| `ADMIN_EMAILS` | none | comma-separated; they become admins on registration |
| `REQUEST_TIMEOUT_SECONDS` | `10` | |
| `LOG_FORMAT` | `pretty` | or `json` |
| `RUST_LOG` | see `telemetry.rs` | environment only, e.g. `RUST_LOG=debug` |

- [`Config::from_sources(file_text, lookup)`](src/config.rs) is a **pure function**: it takes the file's text and a "look up a variable" closure. `Config::load()` passes the real ones; tests and the `config_layers` example pass made-up ones. Nothing calls `std::env::set_var`, which is `unsafe` in Rust 2024 because changing the environment while other threads read it is undefined behaviour. The binary's own end-to-end test sets variables **on the child process** instead.
- `Secret` wraps the JWT secret with a `Debug` that prints `[redacted]`, so `tracing::info!(?config)` at start-up is safe.
- [`.env.example`](.env.example) documents every variable with example values only; a unit test checks it parses. Real `.env` files are ignored by Git ([`.gitignore`](.gitignore)).

The real output of `config_layers` (abridged):

```text
== the environment on top (it wins)
port 7000 | db sqlite:from-the-file.db | admins ["ama@example.com"]

== Debug output never shows the secret
Config { host: 127.0.0.1, port: 7000, database_url: "sqlite:from-the-file.db", jwt_secret: Secret("[redacted]"), … }

== mistakes are caught before the server starts
error: DATABASE_URL is not set (see .env.example)
error: JWT_SECRET = "[redacted]": must be at least 32 bytes long
error: TOKEN_TTL_SECONDS = "an hour": invalid digit found in string
error: LOG_FORMAT = "xml": must be `pretty` or `json`
```

> 💡 **Analogy: settings like layers of clothes.** The defaults are your vest, always on. The config file is the jumper your parents laid out. The environment is the coat you grab at the door, and it goes on top of everything. And before leaving the house, someone checks you're not wearing two left shoes (validation at start-up).

▶ `cargo run -p capstone_rest_api --example config_layers`

## 3. The database: SQLx, SQLite and migrations

As in [lesson 06/03](../03_sqlx/README.md): a pool, `sqlx::migrate!()` with [`migrations/`](migrations) run at start-up, bind parameters everywhere, `QueryBuilder` for the list filters, and constraints (`UNIQUE`, `CHECK`, `REFERENCES … ON DELETE CASCADE`) as the last line of defence. Three things are new:

- **One filter function for two queries.** The page (`LIMIT/OFFSET`) and the count (`COUNT(*)`) share `push_filters`, so `total` always matches the items.
- **Partial updates in one statement:** `SET rating = COALESCE(?, rating)` binds `NULL` for a missing field, which keeps the old value.
- **Runtime-checked queries, fully covered by tests.** The `query!` macros would also work (with a committed `.sqlx/`, lesson 06/03); for a service, adding them and `cargo sqlx prepare --check` in CI is a good next step.

SQLite is a fine production database for one server process. When you need several replicas, switch to PostgreSQL: the `postgres` feature, `PgPool`, `$1` placeholders, `BIGINT GENERATED ALWAYS AS IDENTITY`, and `TIMESTAMPTZ` instead of text timestamps. The layers above the repository don't change.

> 💡 **Analogy: the archive.** Only the archivists go into the stacks, and they follow numbered building plans (migrations) to put up new shelves. Lists are handed out a page at a time, and the count on the cover always matches the pages inside, because both were made with the same search slip.

## 4. Passwords: Argon2id

```rust
let passwords = Passwords::new(19 * 1024, 2)?;           // m = 19 MiB, t = 2 passes, p = 1
let phc = passwords.hash("correct horse battery")?;     // a fresh random salt every time
passwords.verify("correct horse battery", &phc);        // true
```

- Never store passwords, and never a **fast** hash of them: a GPU tries billions of SHA-256 guesses a second. **Argon2id** (winner of the Password Hashing Competition, recommended by OWASP) is slow and needs a lot of memory **on purpose**, which makes each guess expensive. OWASP suggests at least 19 MiB, 2 iterations, parallelism 1, which is also the `argon2` crate's default.
- Each hash gets a random **salt**, so equal passwords get different hashes and precomputed tables are useless.
- The result is a **PHC string** that records the algorithm, version, costs and salt, so `verify` needs nothing else, and old hashes keep working after you raise the cost.
- Hashing is CPU work that takes a while (real output below: 480 ms in a debug build, 37 ms measured in `--release`), so the service runs it with **`tokio::task::spawn_blocking`**: on an async worker thread it would stall every other request there.
- **Logins don't reveal which e-mails exist.** An unknown e-mail still runs a hash check against a dummy hash (`verify_dummy`), so both failures take as long and give the same message.
- Tests and examples use the minimum cost (8 KiB, 1 pass) through config, so the suite stays fast; `PASSWORD_MEMORY_KIB` must never be that low in production.

The real output of `password_hashing` (debug build):

```text
production cost: hashed in   480.45ms
           $argon2id$v=19$m=19456,t=2,p=1$eXH4XT5p8xqmfFwXqm/RDw$ZifuA6z8CqUdw6awQgAKPaQw84Lz+IPfVhTjTdq5qBI
tests      cost: hashed in     1.66ms
           $argon2id$v=19$m=8,t=1,p=1$COR/S7Aa/eeeqono5kipQw$sdA9or208d7MT7qW8ffQ1nFnPsXgwuHLWrSOgbrteX4

same password, different salts: true
right password verifies:        true
wrong password verifies:        false
```

> 💡 **Analogy: a fingerprint of your PIN, made in a slow machine.** The library never writes your PIN down. It feeds the PIN, plus a random pinch of glitter (the salt), through a machine that takes a whole second and fills a big room while it works (the cost), and keeps only the printout. To check you, it runs your PIN through the same machine with the same glitter and compares printouts. A thief who steals the printouts must run the slow machine once for every guess, for every person.

▶ `cargo run -p capstone_rest_api --example password_hashing`

## 5. Tokens: JSON Web Tokens

After a login, the client sends `Authorization: Bearer <token>` with every request. A JWT is `header.claims.signature`, each part base64url:

```text
header:    {"typ":"JWT","alg":"HS256"}
claims:    {"sub":"42","role":"member","iss":"capstone-rest-api","iat":1794669698,"exp":1794673298}
signature: yj0lGqli8_BxsgR6zlf81vDxdWkxGv1JOTc61WQekU4 (HMAC-SHA256 of the first two parts)
```

- Anyone can **read** the claims; only the holder of `JWT_SECRET` can **sign** them. The server checks the signature and trusts the claims without a database lookup: that's what makes tokens **stateless**. Never put secrets in claims.
- [`Tokens`](src/auth/token.rs) pins the algorithm to **HS256** (a server that lets the token's header choose the algorithm can be tricked), requires `exp`, `iss` and `sub`, checks the issuer, and allows 5 seconds of clock difference.
- The price of statelessness: a token stays valid until it expires, even after the user is deleted or demoted. Keep the lifetime short (`TOKEN_TTL_SECONDS`), and add refresh tokens or a deny-list when you need logout or instant revocation.
- **Why `jsonwebtoken` with `rust_crypto`:** `jsonwebtoken` 12 needs exactly one crypto backend chosen by feature. `aws_lc_rs` builds AWS-LC from C and assembly; `rust_crypto` uses the pure-Rust RustCrypto crates (`hmac`, `sha2`, `rsa`, `p256`, …). This project uses `default-features = false, features = ["rust_crypto"]`: it builds anywhere Rust does, and `cargo tree -i cc -p capstone_rest_api` shows that the only C code in the whole program is SQLite's.

The real output of `jwt_tokens`: promote yourself to admin by editing the claims, and the signature gives you away:

```text
verify(valid)       -> Ok("42")
forged claims: {"sub":"42","role":"admin","iss":"capstone-rest-api","iat":1794669698,"exp":1794673298}
verify(edited)      -> Err(Unauthorized("the token is not valid"))
verify(other key)   -> Err(Unauthorized("the token is not valid"))
verify(expired)     -> Err(Unauthorized("the token has expired"))
```

> 💡 **Analogy: a library card with a wax seal.** The card says your name and that you're a member, in plain writing that anyone can read. What makes it real is the library's **wax seal** (the signature), which only the library's own stamp (the secret) can make. Scratch out "member" and write "admin", and the seal no longer matches. And every card has a date on it, after which it's no good.

▶ `cargo run -p capstone_rest_api --example jwt_tokens`

## 6. Authentication: the `AuthUser` extractor

```rust
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;          // a 401 in our JSON format

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let header = parts.headers.get(AUTHORIZATION)
            .ok_or(AppError::Unauthorized("missing Authorization header"))?;
        let token = header.to_str().ok()
            .and_then(|value| value.strip_prefix("Bearer "))
            …;
        let claims = state.auth.verify_token(token)?;
        Ok(AuthUser { id: claims.sub.parse()…, role: claims.role })
    }
}
```

A handler that lists `user: AuthUser` among its arguments only runs for authenticated requests; one that doesn't is public. Axum 0.8 extractors are plain `async fn`s in the trait (no `#[async_trait]`). Every 401 also carries `WWW-Authenticate: Bearer`, which tells clients how to authenticate (RFC 9110). Exercise 5 builds this extractor from scratch.

> 💡 **Analogy: the card reader on a door.** Doors to the reading rooms have a card reader; the hall doesn't. You don't show your card to every librarian: the reader at the door checks it once, and the people inside know you're allowed in.

## 7. Authorisation: a policy, not scattered `if`s

Authentication says **who** you are (401 when it fails). Authorisation says what you **may do** (403). The rules live in one pure function, [`service/policy.rs`](src/service/policy.rs):

```rust
pub fn may(actor: &AuthUser, _action: Action, author_id: i64) -> bool {
    match actor.role {
        Role::Admin => true,                    // admins may edit and delete anything
        Role::Member => actor.id == author_id,  // members only their own
    }
}
```

The service calls `authorize(actor, Action::Edit, author)?` before changing anything, and the test checks every role × action × owner combination. Roles come from the token; admins are the addresses in `ADMIN_EMAILS` at registration. A stricter API answers **404 instead of 403** for things a user may not even know exist; for public reviews, 403 is honest. Exercise 6 writes a richer policy (moderators, suspended users, locked posts).

> 💡 **Analogy: the rulebook on the back-office wall.** The card reader let you into the building; the rulebook says what you may do inside: "Members may cross out their own notes. Head librarians may cross out anyone's." One page, pinned up where everyone can read it, instead of every clerk remembering their own version.

## 8. Validation, and one error format for everything

[`AppError`](src/error.rs) is the single error type. Every layer converts into it (`From<sqlx::Error>`, `From<JsonRejection>`, …), so `?` works everywhere, and `IntoResponse` decides the status and body:

| Variant | Status | `code` | Raised by |
|---|---|---|---|
| `Validation(Vec<FieldError>)` | 422 | `validation_failed` (+ `details`) | services, via `validate()` |
| `BadRequest` | 400 | `bad_request` | services (e.g. `min_rating=9`) |
| `Rejected { status, .. }` | axum's: 400, 413, 415, 422 | `bad_request`, `payload_too_large`, `unsupported_media_type`, `invalid_body` | `AppJson`, `AppQuery`, `AppPath` |
| `Unauthorized` | 401 + `WWW-Authenticate` | `unauthorized` | the extractor, login |
| `Forbidden` | 403 | `forbidden` | the policy |
| `NotFound` / `MethodNotAllowed` | 404 / 405 | `not_found` / `method_not_allowed` | services, router fallbacks |
| `Conflict` | 409 | `conflict` | duplicate e-mail (a `UNIQUE` violation) |
| `Internal` | 500 | `internal` | anything unexpected: the details are **logged**, the client gets "something went wrong on our side" |

```json
{"error": {"code": "validation_failed", "message": "the request has invalid fields",
           "details": [{"field": "rating", "message": "must be from 1 to 5"}]}}
```

- **Validation reports every problem at once** (so a form can show them all), trims titles and names, and lower-cases e-mails before they're compared or stored.
- **Axum's own rejections in our format:** `AppJson<T>` is `#[derive(FromRequest)] #[from_request(via(axum::Json), rejection(AppError))]`: it behaves exactly like `Json<T>`, but a bad body becomes an `AppError`. The same trick gives `AppQuery` and `AppPath`, and `fallback` / `method_not_allowed_fallback` cover unknown routes.
- `code` is for programs (stable, never reworded); `message` is for people.

> 💡 **Analogy: one complaints form for the whole library.** Whether the problem was at the desk, in the back office or in the stacks, the visitor gets the same kind of slip: a reason code in a box at the top, a sentence underneath, and a list of what to fix. If the boiler exploded in the basement, the slip just says "sorry, our fault"; the details go in the caretaker's logbook, not to the visitor.

## 9. Pagination

`GET /api/v1/reviews?page=2&per_page=10` returns:

```json
{"items": [...], "page": 2, "per_page": 10, "total": 57, "total_pages": 6}
```

- Missing or nonsense parameters are clamped, not refused: `page` defaults to 1, `per_page` to 20, at most 100, so nobody can ask for a million rows.
- `OFFSET = (page - 1) × per_page`, computed in `u64`; `total_pages = ⌈total / per_page⌉` (`div_ceil`).
- The order must be **total and stable**: `ORDER BY created_at DESC, id DESC`. Without the `id` tie-breaker, rows with equal timestamps could swap places between pages.
- Offset pagination re-counts and skips rows, which gets slow deep into a big table, and new rows shift the pages. **Cursor (keyset) pagination** (`?after=<last id>`, `WHERE id < ?`) avoids both; it's the usual next step.

> 💡 **Analogy: a book of results.** Instead of tipping every card in the catalogue onto the desk, the librarian hands you page 2 of 6, ten cards per page, and always sorts them the same way, so a card can't appear on two pages or on none.

## 10. Structured logging and middleware

```rust
tracing::info!(review_id = review.id, user_id = actor.id, "review created");
```

`tracing` records **events with typed fields** inside **spans**. `TraceLayer` opens a span per request carrying the method, path and `request_id`, so every line logged while handling that request carries them too. The real log of the session above (`LOG_FORMAT=pretty`):

```text
INFO request{method=POST path=/api/v1/auth/register request_id=2}: capstone_rest_api::service::auth: registered user_id=1 role="member"
INFO request{method=POST path=/api/v1/auth/register request_id=2}: tower_http::trace::on_response: finished processing request latency=697 ms status=201
INFO request{method=POST path=/api/v1/auth/login request_id=3}: capstone_rest_api::service::auth: logged in user_id=1
INFO request{method=POST path=/api/v1/reviews request_id=4}: capstone_rest_api::service::reviews: review created review_id=1 user_id=1
INFO request{method=DELETE path=/api/v1/reviews/1 request_id=6}: tower_http::trace::on_response: finished processing request latency=0 ms status=401
```

(697 ms for a registration: production-cost Argon2 in a debug build.) With `LOG_FORMAT=json` the same events are one JSON object per line, ready for a log collector. The `structured_logging` example shows both:

```text
{"timestamp":"2026-11-14T15:22:01.710972Z","level":"WARN","fields":{"message":"password too short","length":5},"target":"structured_logging","span":{"email":"kofi@example.com","name":"login"},"spans":[{"method":"POST","request_id":7,"name":"request"},{"email":"kofi@example.com","name":"login"}]}
```

Log **ids and outcomes**, never passwords, tokens or secrets (`#[instrument(skip(password))]`).

The middleware stack in [`routes.rs`](src/routes.rs), outermost first (`ServiceBuilder` reads top to bottom):

| Layer | Does |
|---|---|
| `SetRequestIdLayer` | gives each request an `x-request-id` (keeps one a proxy already set) |
| `TraceLayer` | one span per request; logs every response with its status and latency |
| `PropagateRequestIdLayer` | copies `x-request-id` onto the response, so a client can quote it in a bug report |
| `TimeoutLayer` | answers 408 after `REQUEST_TIMEOUT_SECONDS` |
| `DefaultBodyLimit` | refuses bodies over 64 KiB (413) |

> 💡 **Analogy: the day book and the ticket machine.** Every visitor pulls a numbered ticket at the door (the request id). Everything the staff write in the day book about that visit has the ticket number next to it, in neat columns (fields), so later you can find every line about visit 4 in one search. The doorman also times each visit, and politely shows out anyone who stays too long (the timeout).

▶ `cargo run -p capstone_rest_api --example structured_logging` (add `-- json` for JSON)

## 11. Testing at every level

| Level | Where | How | Count |
|---|---|---|---|
| Pure logic | `src/**` `#[cfg(test)]` | config, validation, pagination, policy, password, token, error, client | 23 |
| Handlers | [`tests/api.rs`](tests/api.rs) | the full router + in-memory SQLite through `tower::ServiceExt::oneshot` (no network) | 12 |
| Over TCP | [`tests/e2e.rs`](tests/e2e.rs) | `serve` on `127.0.0.1:0`, a real HTTP session, then graceful shutdown | 1 |
| The binary | [`tests/e2e.rs`](tests/e2e.rs) | runs `target/…/capstone_rest_api` with **only environment variables set on the child**, reads its JSON log for the port, calls `/health`, runs the `healthcheck` subcommand, then kills it; and checks it refuses to start without `JWT_SECRET` (exit code 2) | 2 |

- Each handler test builds a fresh app with `Config::for_tests()`: its own in-memory database, cheap Argon2, a fixed fake secret. They run in parallel and the whole suite takes well under a second.
- The binary test kills its child in a `Drop` guard, so even a failing test leaves no server running.

> 💡 **Analogy: checking a library before opening day.** You test each rule on paper (unit tests), walk through every department with a pretend visitor (handler tests), open the doors for one real visitor (TCP), and finally check that the building opens with the keys you'll actually be given (the binary with its environment).

## 12. Graceful shutdown and health checks

- `serve(listener, app, shutdown_signal())` stops accepting connections when **Ctrl+C** or **SIGTERM** arrives (SIGTERM is what `docker stop` and Kubernetes send first), then waits for requests in flight.
- `GET /health` runs `SELECT 1`: 200 when the database answers, 503 when it doesn't, so a load balancer stops sending traffic to a broken instance.
- `capstone_rest_api healthcheck` is the same check as a command (exit 0 or 1), because the slim runtime image has no `curl`.

> 💡 **Analogy: closing time, and the morning check.** At closing time the doors lock but everyone already inside may finish their book. Every morning, the caretaker tries the archive door (`SELECT 1`) before turning the sign to OPEN.

## 13. Deployment: Docker and docker-compose

[`Dockerfile`](Dockerfile) is a **multi-stage** build:

1. `rust:1-slim-bookworm` copies the workspace and runs `cargo build --release --locked -p capstone_rest_api`. BuildKit **cache mounts** keep the crates.io registry and `target/` between builds, so a code change doesn't recompile every dependency. (The workspace's `rust-toolchain.toml` makes rustup fetch the current stable Rust; the image's `gcc` compiles the bundled SQLite.)
2. `debian:bookworm-slim` gets **only the binary**: no compiler, no source. It runs as an unprivileged user, keeps the database in a `/data` volume, sets container defaults (`APP_HOST=0.0.0.0`, `LOG_FORMAT=json`), declares a `HEALTHCHECK`, and starts the server as PID 1 so it receives SIGTERM.

`JWT_SECRET` is **not** in the image: anyone who can pull an image can read its environment. It is supplied when the container starts.

The crate is a workspace member, so the build context is the **workspace root**; [`Dockerfile.dockerignore`](Dockerfile.dockerignore) (BuildKit reads `<Dockerfile>.dockerignore` next to the Dockerfile) keeps `target/`, `.git` and any `.env` out of it. [`docker-compose.yml`](docker-compose.yml) builds it, reads secrets from `.env`, publishes the port on `127.0.0.1` only, mounts a named volume for `/data`, makes the container filesystem read-only, and gives 15 seconds for a graceful stop:

```bash
cd 06_framework_mastery/07_capstone_rest_api
cp .env.example .env            # then put a long random JWT_SECRET in it
docker compose up --build       # http://127.0.0.1:8080/health
docker compose down
```

(These files were written for this lesson but not run here; the course machine doesn't need Docker. Everything they rely on is tested: the configuration variables, the `0.0.0.0` binding, `healthcheck`, and SIGTERM handling on Unix.)

> 💡 **Analogy: building the library, then moving in.** The builders arrive with cranes, cement mixers and scaffolding (the Rust toolchain). When the building is done, all of that goes away and only the finished library (the binary) is handed over. The keys to the safe (secrets) are given to the head librarian on opening day, never cemented into the walls.

## 14. The production checklist

Before real users depend on it:

- [ ] **Secrets:** `JWT_SECRET` is long and random (`openssl rand -base64 48`), comes from a secrets manager or the platform, is never in Git or an image, and can be rotated. The server warns if it still contains `change-me`.
- [ ] **HTTPS:** TLS terminates at a load balancer or reverse proxy (or in the app with `rustls`); plain HTTP never leaves the private network.
- [ ] **Passwords:** Argon2id at OWASP's minimum cost or above (aim for tens of milliseconds in a release build on the real hardware); never the test cost.
- [ ] **Tokens:** short lifetimes; a refresh-token or deny-list design if logout or instant revocation matters; one algorithm, pinned.
- [ ] **Abuse:** rate limits on `/auth/login` and `/auth/register` (at the proxy, or a Tower rate-limit layer), and alerts on bursts of 401s.
- [ ] **Limits:** body size, request timeout, page size and pool size are set deliberately.
- [ ] **CORS:** only if browsers on other origins call the API, with an explicit allow-list (`tower-http`'s `CorsLayer`).
- [ ] **Database:** backups (and a tested restore); migrations reviewed and never edited after release; Postgres before running several replicas; compile-time checked queries in CI.
- [ ] **Errors and logs:** no internal details in responses; JSON logs with request ids shipped somewhere searchable; no secrets or tokens in logs.
- [ ] **Health and shutdown:** `/health` wired to the load balancer or orchestrator; a stop grace period longer than the request timeout.
- [ ] **Container:** non-root user, read-only filesystem, pinned base image versions (or digests), resource limits.
- [ ] **CI:** `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo build --locked`, and a dependency audit (`cargo audit` or `cargo deny`).
- [ ] **Observability:** metrics (request rate, errors, latency) and alerts, not just logs; tracing across services (OpenTelemetry) once there's more than one.
- [ ] **API hygiene:** a version in the path (`/api/v1`), documented responses (an OpenAPI description), and stable error `code`s.

> 💡 **Analogy: the pilot's checklist.** Pilots who have flown for thirty years still read the checklist before every take-off, because the one thing you forget is the one that matters. This list is that card for a web service.

## Common mistakes

| You wrote | What happens | Usually the fix |
|---|---|---|
| `?` on a `sqlx::Error` in a function returning `Result<_, AppError>` | E0277 `?` couldn't convert the error to `AppError` | `impl From<sqlx::Error> for AppError` |
| `app.oneshot(request)` in a test | E0599 no method named `oneshot` found for struct `Router<S>` | `use tower::ServiceExt;` |
| Two response types from one `-> impl IntoResponse` | E0308 `if` and `else` have incompatible types | `.into_response()` on each, return `Response` (or a `Result`) |
| Business rules in a handler | duplicated and untested rules | Move them to a service |
| SQL in a handler or service | the database leaks everywhere | Keep SQL in repositories |
| `#[derive(Serialize)]` on a struct with `password_hash` | the hash goes out in a response | Separate public and stored types (`User` vs `StoredUser`) |
| SHA-256 (or no hash) for passwords | cracked in minutes after a leak | Argon2id with a real cost |
| Argon2 on an async worker thread | every request on that thread stalls | `tokio::task::spawn_blocking` |
| "No such user" vs "wrong password" | attackers learn which e-mails exist | One message, and a dummy verify |
| Trusting the token's `alg` header | forged tokens accepted | `Validation::new(Algorithm::HS256)` only |
| Secrets in claims | anyone can read them | Claims are public: ids and roles only |
| `JWT_SECRET` in the Dockerfile or Git | everyone with the image or repo has it | Supply it at run time; ignore `.env` |
| `std::env::set_var` in tests | `unsafe` in Rust 2024, races between threads | Pass config explicitly; set env on a child `Command` |
| Tests at production Argon2 cost | a slow suite (≈0.5 s per hash in debug) | Cheap cost from `Config::for_tests()` |
| `ORDER BY created_at` alone for pages | rows repeat or vanish between pages | Add a unique tie-breaker (`id`) |
| Leaking `sqlx::Error` text in a 500 | attackers learn your schema | Log it; send a generic message |
| Binding to `127.0.0.1` in a container | unreachable from outside the container | `APP_HOST=0.0.0.0` |
| Shell-form `CMD capstone_rest_api` in a Dockerfile | the server doesn't get SIGTERM; `docker stop` kills it | Exec form `CMD ["capstone_rest_api"]` |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p capstone_rest_api --example config_layers
cargo run -p capstone_rest_api --example password_hashing
cargo run -p capstone_rest_api --example jwt_tokens
cargo run -p capstone_rest_api --example layers_in_action
cargo run -p capstone_rest_api --example structured_logging
cargo run -p capstone_rest_api --example api_tour
```

**2. Do the exercises** in `exercises/`. Each is a small, self-contained version of one piece of the capstone:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_question_mark_conversion` | Make it compile (E0277): `From` between layers | `cargo run -p capstone_rest_api --features exercises --bin ex01_fix_question_mark_conversion` |
| 2 | `ex02_fix_missing_service_ext` | Make it compile (E0599): test a router with `oneshot` | `cargo run -p capstone_rest_api --features exercises --bin ex02_fix_missing_service_ext` |
| 3 | `ex03_fix_incompatible_branches` | Make it compile (E0308): two responses, one type | `cargo run -p capstone_rest_api --features exercises --bin ex03_fix_incompatible_branches` |
| 4 | `ex04_pagination` | Defaults, clamping, offsets, page counts; pass 4 tests | `cargo test -p capstone_rest_api --features exercises --bin ex04_pagination` |
| 5 | `ex05_bearer_extractor` | A `FromRequestParts` authentication extractor; pass 4 tests | `cargo test -p capstone_rest_api --features exercises --bin ex05_bearer_extractor` |
| 6 | `ex06_authorization_policy` | A four-role policy as one pure function; pass 4 tests | `cargo test -p capstone_rest_api --features exercises --bin ex06_authorization_policy` |

The real errors for the first three:

```text
error[E0277]: `?` couldn't convert the error to `AppError`
  --> 06_framework_mastery\07_capstone_rest_api\exercises\ex01_fix_question_mark_conversion.rs:31:43
   |
31 |     let title = find_title(pool, id).await?;
   |                 --------------------------^ the trait `From<sqlx::Error>` is not implemented for `AppError`
   |                 |
   |                 this can't be annotated with `?` because it has type `Result<_, sqlx::Error>`
```

```text
error[E0599]: no method named `oneshot` found for struct `Router<S>` in the current scope
   --> 06_framework_mastery\07_capstone_rest_api\exercises\ex02_fix_missing_service_ext.rs:27:24
    |
 27 |     let response = app.oneshot(request).await.unwrap();
    |                        ^^^^^^^
...
help: trait `ServiceExt` which provides `oneshot` is implemented but not in scope; perhaps you want to import it
    |
 13 + use tower::util::ServiceExt;
```

```text
error[E0308]: `if` and `else` have incompatible types
  --> 06_framework_mastery\07_capstone_rest_api\exercises\ex03_fix_incompatible_branches.rs:31:9
   |
28 | /     if id == 1 {
29 | |         (StatusCode::OK, Json(json!({ "id": 1, "title": "Emma" })))
   | |         ----------------------------------------------------------- expected because of this
30 | |     } else {
31 | |         StatusCode::NOT_FOUND
   | |         ^^^^^^^^^^^^^^^^^^^^^ expected `(StatusCode, Json<Value>)`, found `StatusCode`
32 | |     }
   | |_____- `if` and `else` have incompatible types
```

(For exercise 3 the compiler also suggests `Box<dyn IntoResponse>`; that only leads to E0277 about `Handler`.)

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p capstone_rest_api --features solutions --bins`).

**4. Extend the capstone.** Each of these touches several layers; let the tests guide you:

- `GET /api/v1/users/{id}/reviews`: a repository filter, a service method, a handler and a route.
- A `moderator` role (exercise 6): a migration that widens the `CHECK`, the `Role` enum, the policy, and tests.
- Refresh tokens: a `refresh_tokens` table, `POST /api/v1/auth/refresh`, and a short access-token lifetime.
- Cursor pagination: `?after=<id>` with `WHERE r.id < ?`.
- Switch the database to PostgreSQL (`docker run --rm -e POSTGRES_PASSWORD=change-me -p 5432:5432 postgres:17`), changing only `state.rs`, the repositories and the migrations.

```bash
cargo test -p capstone_rest_api   # 23 unit + 12 handler + 3 end-to-end tests
```

## Checklist

- [ ] Layered architecture: routes → handlers → services → repositories, and what each must not know
- [ ] Configuration from defaults, a file and the environment; validated at start-up; secrets redacted; no `set_var`
- [ ] SQLx + SQLite: pool, migrations at start-up, filters shared by page and count, `COALESCE` patches
- [ ] Argon2id: salts, PHC strings, cost, `spawn_blocking`, equal-time failures
- [ ] JWT: structure, HS256 pinned, `exp`/`iss`/`sub`, statelessness and its cost; the pure-Rust backend choice
- [ ] Authentication with a `FromRequestParts` extractor; 401 with `WWW-Authenticate`
- [ ] Authorisation with a pure policy; 403 (or 404) on refusal
- [ ] Validation that reports every problem; one JSON error shape; `Internal` hides details; custom `AppJson`/`AppQuery`/`AppPath`
- [ ] Pagination: clamping, `LIMIT`/`OFFSET`, `total_pages`, a stable order; cursors as the next step
- [ ] `tracing`: events, fields, spans, request ids, JSON output; the middleware stack and its order
- [ ] Tests: unit, `oneshot` handler tests, TCP on port 0, the real binary with env on a child process
- [ ] Graceful shutdown (Ctrl+C, SIGTERM) and a health check
- [ ] Multi-stage `Dockerfile`, `.dockerignore`, `docker-compose.yml`, secrets at run time
- [ ] The production checklist
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- A service is easier to change, test and trust in **layers**: routes and handlers speak HTTP, services hold the rules, repositories hold the SQL.
- **Configuration** is loaded in layers and checked once; secrets never reach logs, Git or images.
- **Passwords** are stored only as slow, salted Argon2id hashes; **tokens** are signed claims, checked with one pinned algorithm, and short-lived.
- An **extractor** authenticates (401); a **policy** authorises (403).
- One **error type** and one JSON shape for every failure; validation reports everything at once; internal errors stay internal.
- **Lists** are paged with stable ordering; **logs** are structured, per-request, and searchable.
- **Tests** cover pure logic, handlers without a network, the server over TCP, and the real binary configured like production.
- A **multi-stage image** ships only the binary, as a non-root user, with secrets supplied at run time and a **checklist** before go-live.

**In one sentence for a child:** we built a whole library where you sign up with a secret PIN that nobody ever writes down, get a sealed library card, and can share what you thought of books, while the signposts, the desk clerks, the rulebook and the archivists each do their own job and write everything neatly in the day book.

**Next:** that's the end of the course: you went from `cargo new` and "Hello, world!" all the way to a tested, deployable web service. Go back to the [course map](../../README.md) to revisit any lesson, and keep building.
