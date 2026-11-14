<!-- hand-written lesson: gen_course must not overwrite -->
# Actix-web: the other big Rust web framework

**Docs:** [actix.rs guide](https://actix.rs/docs/) · [API docs](https://docs.rs/actix-web) · [examples repository](https://github.com/actix/examples) · [`actix-web` on crates.io](https://crates.io/crates/actix-web)
· Runtime: [`actix-rt`](https://docs.rs/actix-rt), [`actix-server`](https://docs.rs/actix-server)
· Compare with: [lesson 06/01, Axum](../01_axum/README.md)

Run it: `cargo run -p actix_web_basics`, then open <http://127.0.0.1:8080>  ·  Test it: `cargo test -p actix_web_basics`

[Actix-web](https://actix.rs) is, with Axum, one of the two most used Rust web frameworks. It is older, very fast, and has its own way of doing things: route macros such as `#[get("/")]`, `App` and `HttpServer` builders, and its own runtime that runs one single-threaded event loop per CPU core. You already know Axum from [lesson 06/01](../01_axum/README.md), so this lesson builds **the same notes JSON API** again, and points out every place where the two frameworks agree or differ.

After this lesson you will be able to:

- start a server with `HttpServer` and `App`, and stop it gracefully;
- write handlers with `#[get]`/`#[post]` and wire them with `route`, `resource` and `scope`;
- take requests apart with extractors (`web::Path`, `web::Query`, `web::Json`, `web::Data`…);
- share state between worker threads correctly with `web::Data`;
- return anything that implements `Responder`, and errors with your own `ResponseError`;
- add middleware (`Logger`, `wrap_fn`, `from_fn`) and scope it;
- test handlers without a network (`actix_web::test`), and the real server on a free port;
- explain Actix's actor heritage and why its handlers don't need to be `Send`;
- read an Axum handler and write the Actix one, and the other way round.

> 💡 **The big picture: a restaurant with several kitchens.** Axum is one big kitchen where any cook can pick up any order, even half-finished ones (Tokio's work-stealing). Actix builds **several small kitchens** (workers), each with **one cook** who looks after their own orders from start to finish. `HttpServer` is the **restaurant manager**: it opens the door (binds a port), hires a cook per kitchen, and gives each the same **recipe book** (your `App` factory). Because an order never leaves its kitchen, a cook may use their own non-shareable tools (`Rc`). Anything **all** kitchens must share, like the till (`web::Data`), has to be safe to share.

## Before you start: Actix-web next to Axum, and its Cargo features

| | Axum (06/01) | Actix-web (this lesson) |
|---|---|---|
| Built on | Tokio + Tower + hyper | its own `actix-rt` (on Tokio), `actix-server`, `actix-http` |
| Handler | any `async fn` whose arguments are extractors | the same, plus optional route macros `#[get("/")]` |
| Wiring | `Router::new().route("/", get(h))` | `App::new().service(h)` or `.route("/", web::get().to(h))` |
| Shared state | `State<T>`, `T: Clone` (often `Arc`) | `web::Data<T>` (an `Arc` inside) |
| Response | `impl IntoResponse` | `impl Responder` |
| Errors | `impl IntoResponse for MyError` | `impl ResponseError for MyError` |
| Middleware | Tower `Layer`s, `middleware::from_fn` | `Transform`s, `wrap_fn`, `middleware::from_fn` |
| Runtime model | one multi-threaded work-stealing runtime | one single-threaded runtime per worker thread |
| Handler futures must be `Send` | yes | no |
| Test without network | `tower::ServiceExt::oneshot` | `actix_web::test::init_service` + `call_service` |

```toml
[dependencies]
actix-web = { version = "4.19.0", default-features = false, features = ["macros"] }
```

Actix-web's **default features** add response compression (brotli, gzip and zstd), cookies, HTTP/2, WebSockets and Unicode-aware routing. `compress-zstd` builds the zstd **C library** (through `zstd-sys`, which compiles it with the `cc` crate), so it needs a C compiler. This lesson only needs `macros` (for `#[get]`, `#[actix_web::main]` and `#[actix_web::test]`), so it switches the defaults off: no C compiler needed, and a much smaller build. (On this course's Windows machine the MSVC compiler is available and the defaults would build too; switching them off is about not paying for what you don't use.) Add features back when you want them, e.g. `features = ["macros", "compress-gzip", "cookies"]`. Actix-web 4.19 requires Rust 1.89 or newer.

> 💡 **Analogy: ordering a kitchen.** The default kitchen comes with a pizza oven, an ice-cream machine and a smoker. If you only make sandwiches, order the kitchen **without** them: it's delivered sooner, and the smoker (which needs its own gas supply, the C compiler) doesn't need connecting.

## 1. The first program: `HttpServer`, `App` and workers

```rust
use actix_web::{App, HttpServer, Responder, get};

#[get("/")]
async fn hello() -> impl Responder {
    "Hello, world!"
}

#[actix_web::main]                       // like #[tokio::main], for Actix's runtime
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hello))   // a FACTORY, called once per worker
        .bind(("127.0.0.1", 8080))?
        .run()                           // a `Server`: a future that runs until stopped
        .await
}
```

- **`App`** is your application: routes, data, middleware. **`HttpServer`** runs it: it binds the socket, starts **worker threads** (by default `std::thread::available_parallelism()` of them), and calls your closure **once per worker** so each has its own `App`.
- `bind` takes an address; `listen` takes a `std::net::TcpListener` you bound yourself, which is how the tests use port 0.
- `run()` returns a `Server` future. `server.handle()` gives a `ServerHandle`: `handle.stop(true).await` stops **gracefully** (stop accepting, finish requests in flight, by default for up to 30 seconds: `shutdown_timeout`). Actix also catches Ctrl+C by itself and shuts down the same way (`disable_signals()` turns that off).

The real output of `hello_server`, which starts, asks itself two questions and stops:

```text
listening on http://127.0.0.1:6578
GET /           -> 200 "Hello, world!"
GET /greet/Ama  -> 200 "Hello, Ama!"
server stopped
```

> 💡 **Analogy: the restaurant manager.** `HttpServer` is the manager who unlocks the door (`bind`), opens a few kitchens (workers) and hands each the **same recipe book** to copy (the factory closure). `App` is the recipe book. `stop(true)` is closing time: the door is locked, the cooks finish the plates already on the stove, then everyone goes home.

▶ `cargo run -p actix_web_basics --example hello_server`

## 2. Routes: macros, resources, scopes and guards

```rust
#[get("/books")]                                    // 1. a route macro on the handler
async fn list_books() -> impl Responder { "all books" }

App::new()
    .service(list_books)                            //    …registered with `service`
    .route("/health", web::get().to(health))        // 2. path + method + plain async fn
    .service(
        web::resource("/books/{id}")                // 3. one path, several methods
            .route(web::get().to(one_book))
            .route(web::delete().to(delete_book)),
    )
    .service(
        web::scope("/admin")                        // 4. a prefix, with its own guards,
            .guard(guard::Header("x-admin", "yes")) //    data and middleware
            .route("/stats", web::get().to(admin_stats)),
    )
    .default_service(web::to(not_found))            // 5. when nothing matched
```

- Route macros exist for every method: `#[get]`, `#[post]`, `#[put]`, `#[patch]`, `#[delete]`, `#[head]`, `#[options]`, `#[trace]`, and `#[route("/", method = "GET", method = "POST")]`.
- Path parameters are written `{id}`, like Axum 0.8. `{tail:.*}` matches the rest of the path.
- A **guard** is an extra condition (a header, a host, a method); if it fails, matching moves on.
- **404 or 405?** A `web::resource` with several `.route`s answers an unmatched method with **405**. Separate `#[get("/x")]` and `#[delete("/x")]` macros are separate resources, so `PUT /x` matches neither and falls through to the default service: **404**. Axum answers 405 in both cases. (The mini project's tests check its JSON 404 for `PUT /api/notes/1`.)

The real output of `routing`:

```text
GET    /books                 -> 200 "all books"
POST   /books                 -> 201 "added"
GET    /books/7               -> 200 "book 7"
DELETE /books/7               -> 200 "deleted book 7"
PUT    /books/7               -> 405 ""
GET    /books/seven           -> 404 "can not parse \"seven\" to a u32"
GET    /admin/stats           -> 404 "nothing here"
GET    /admin/stats (x-admin) -> 200 "admin stats"
GET    /nope                  -> 404 "nothing here"
```

> 💡 **Analogy: signs in a department store.** Each route is a sign: "Books, ground floor". A **scope** is a whole floor ("everything upstairs is /admin"), a **guard** is the staff-only door that only opens with a badge, and the **default service** is the information desk that helps anyone who got lost.

▶ `cargo run -p actix_web_basics --example routing`

## 3. Extractors: taking the request apart

Exactly as in Axum, a handler's arguments say what it needs, and each argument type knows how to pull itself out of the request:

| Extractor | Gets | When it fails (Actix's default) | Axum's default |
|---|---|---|---|
| `web::Path<T>` | `{…}` segments, into a tuple or a struct | **404** | 400 |
| `web::Query<T>` | the `?a=1&b=2` query string | 400 | 400 |
| `web::Json<T>` | a JSON body (needs `content-type: application/json`) | 400 (wrong content type too), 413 too big | 422 wrong shape, 400 syntax, 415 content type |
| `web::Form<T>` | an HTML form body | 400 (415 wrong content type) | see 06/01 |
| `web::Data<T>` | shared state (section 4) | 500 "Requested application data is not configured correctly" | compile error (`State` is checked) |
| `web::ReqData<T>` | a value a middleware stored for this request | 500 | `Extension<T>`: 500 |
| `HttpRequest` | method, headers, URI, peer address, app data | never | `Request`, `HeaderMap`, … |

- Any number of extractors may appear, in any order, but only **one** may read the body (`Json`, `Form`, `String`, `Bytes`, `web::Payload`).
- Settings (limits, custom errors) are **app data**: `web::JsonConfig::default().limit(16 * 1024).error_handler(…)`, and the same for `PathConfig` and `QueryConfig`. `JsonConfig`'s default limit is 2 MB. The mini project uses these error handlers to answer every extractor failure in its own JSON format, and to make a bad path id a 400.

The real output of `extractors` (abridged):

```text
GET /users/ama/posts/7           -> 200 tuple: user=ama post=7
GET /v2/users/ama/posts/7        -> 200 struct: user=ama post=7
GET /users/ama/posts/seven       -> 404 can not parse "seven" to a u32
GET /search?page=2               -> 400 Query deserialize error: missing field `q`
POST /users                      -> 201 json: Ama is 9
POST /users                      -> 400 bad JSON: Json deserialize error: invalid value: integer `900`, expected u8 at line 1 column 10
POST /users                      -> 400 bad JSON: Content type error
POST /users                      -> 413 bad JSON: JSON payload (99 bytes) is larger than allowed (limit: 64 bytes)
POST /login                      -> 200 form: q=rust
GET /whoami?q=hi                 -> 200 GET /whoami as "curl/8.5.0", q=hi
```

> 💡 **Analogy: a team of unpackers.** A parcel (the request) arrives. Each argument is an unpacker with one job: one reads the address label (`Path`), one reads the sticky note on top (`Query`), one opens the box (`Json`). If an unpacker finds nonsense, the parcel goes back with a note (the error response) and your handler never sees it. Only one unpacker is allowed to open the box.

▶ `cargo run -p actix_web_basics --example extractors`

## 4. Shared state with `web::Data`, and the per-worker trap

```rust
let state = web::Data::new(AppState::default());     // create it ONCE, outside
HttpServer::new(move || App::new().app_data(state.clone()).configure(routes))

async fn health(state: web::Data<AppState>) -> impl Responder { … }
```

- `web::Data<T>` is an `Arc<T>`: cloning it is cheap and every clone points at the same value. Mutate through `Mutex`, `RwLock` or atomics, exactly as in Axum.
- **The trap:** the factory runs once **per worker**. `App::new().app_data(web::Data::new(Counter::default()))` *inside* the closure gives every worker its **own** counter. That's right for per-worker caches, and wrong for anything shared. The `shared_state` example counts it (real output, 3 workers):

```text
visit ama    -> shared count 1, this worker's count 1
visit kofi   -> shared count 2, this worker's count 1
visit esi    -> shared count 3, this worker's count 1
visit yaw    -> shared count 4, this worker's count 2
visit abena  -> shared count 5, this worker's count 2
visit kwame  -> shared count 6, this worker's count 2
the factory ran 3 times (once per worker)
```

- The factory must own what it uses (it outlives `main`'s stack frame), so borrowing a local fails. The real error from exercise 3:

```text
error[E0373]: closure may outlive the current function, but it borrows `visits`, which is owned by the current function
  --> 06_framework_mastery\05_actix_web\exercises\ex03_fix_factory_borrows_state.rs:38:34
   |
38 |     let server = HttpServer::new(|| App::new().app_data(visits.clone()).service(visit))
   |                                  ^^                     ------ `visits` is borrowed here
   |                                  |
   |                                  may outlive borrowed value `visits`
```

  Adding the suggested `move` then gives E0382 (borrow of moved value: `visits`) where `main` uses it afterwards: move a **clone** into the closure instead.
- Forgetting `.app_data(...)` altogether compiles, then every request using `web::Data<T>` fails at run time with a 500 (and a log line). Axum's `State` is checked at compile time.

> 💡 **Analogy: one till or many?** Every kitchen needs to put money in **the** till. If each kitchen builds its own till when it opens (data made inside the factory), the restaurant's takings are scattered over five tills. Build one till before opening and give each kitchen a **key** to it (`web::Data::clone`).

▶ `cargo run -p actix_web_basics --example shared_state`

## 5. Responders: what a handler may return

A handler returns anything that implements `Responder` (Axum: `IntoResponse`):

| Return | Response |
|---|---|
| `&'static str`, `String` | 200, `text/plain` |
| `web::Json(value)` | 200, `application/json` (any `T: Serialize`) |
| `HttpResponse::Created().insert_header(…).json(…)` | full control: the builder |
| `responder.customize().with_status(…).insert_header(…)` | change any responder's status or headers |
| `Either<A, B>` | one of two responder types |
| `Result<R, E>` | `R`'s response, or `E`'s if `E: ResponseError` (next section) |
| `Option<R>` | `R`'s response, or 404 |
| your type with `impl Responder` | whatever `respond_to` builds |

A type that isn't a responder is a compile error. The real one from exercise 1:

```text
error[E0277]: the trait bound `Book: Responder` is not satisfied
   --> 06_framework_mastery\05_actix_web\exercises\ex01_fix_not_a_responder.rs:25:10
    |
 24 | #[get("/book")]
    | --------------- required by a bound introduced by this call
 25 | async fn book() -> Book {
    |          ^^^^ unsatisfied trait bound
    |
help: the trait `Responder` is not implemented for `Book`
```

The fix is `-> web::Json<Book>`: JSON is a choice you make, not a default.

> 💡 **Analogy: how to hand over the meal.** The kitchen must say **how** a dish leaves: on a plate (`text/plain`), in a takeaway box with a label (`Json`), or specially wrapped by hand (`HttpResponse`). Handing the waiter a bare lasagne with no plate isn't allowed: that's the E0277.

▶ `cargo run -p actix_web_basics --example responders_and_errors`

## 6. Errors: `ResponseError`

```rust
#[derive(Debug)]
enum ShopError { SoldOut(&'static str), Closed }

impl fmt::Display for ShopError { … }          // ResponseError needs Debug + Display

impl ResponseError for ShopError {
    fn status_code(&self) -> StatusCode {        // default: 500
        match self { ShopError::SoldOut(_) => StatusCode::CONFLICT, ShopError::Closed => StatusCode::SERVICE_UNAVAILABLE }
    }
    fn error_response(&self) -> HttpResponse {   // default: the Display text
        HttpResponse::build(self.status_code()).json(json!({ "error": self.to_string() }))
    }
}

#[get("/buy/{item}")]
async fn buy(item: web::Path<String>) -> Result<String, ShopError> { … }   // `?` works
```

- Anything that implements `ResponseError` converts into `actix_web::Error`, so `actix_web::Result<T>` (= `Result<T, actix_web::Error>`) accepts all of them with `?`.
- For quick one-offs, the helpers `error::ErrorBadRequest(e)`, `ErrorNotFound(e)`, `ErrorUnauthorized(e)`… wrap any error with a status.
- Without the impl, `Result<_, MyError>` isn't a responder. The real error from exercise 2:

```text
error[E0277]: the trait bound `LibraryError: ResponseError` is not satisfied
   --> 06_framework_mastery\05_actix_web\exercises\ex02_fix_error_not_response_error.rs:38:10
    |
 37 | #[get("/borrow/{id}")]
    | ---------------------- required by a bound introduced by this call
 38 | async fn borrow(id: web::Path<u32>) -> Result<String, LibraryError> {
    |          ^^^^^^ unsatisfied trait bound
...
    = note: required for `actix_web::Error` to implement `std::convert::From<LibraryError>`
    = note: required for `LibraryError` to implement `Into<actix_web::Error>`
    = note: required for `Result<String, LibraryError>` to implement `Responder`
```

The real output of `responders_and_errors` (abridged):

```text
/customised  -> 418 text/plain; charset=utf-8  teapot
/either/2    -> 404 -                          no such book
/greeting    -> 200 text/html; charset=utf-8   <h1>Hello</h1>
/buy/cake    -> 409 application/json           {"error":"cake is sold out"}
/buy/milk    -> 503 application/json           {"error":"the shop is closed"}
/quick/lots  -> 400 text/plain; charset=utf-8  invalid digit found in string
```

> 💡 **Analogy: the complaints desk.** When something goes wrong, the customer shouldn't get a shrug. `ResponseError` is the **complaints desk script**: "Sold out? Say 409 and explain. Closed? Say 503." Every kitchen uses the same script, so every complaint sounds the same.

## 7. Middleware: `Logger`, `wrap_fn`, `from_fn`

```rust
App::new()
    .wrap_fn(|req, srv| {                         // a closure: before, call, after
        let started = Instant::now();
        let response = srv.call(req);             // needs `use actix_web::dev::Service as _;`
        async move { let r = response.await?; println!("{:?}", started.elapsed()); Ok(r) }
    })
    .wrap(DefaultHeaders::new().add(("x-shop", "open")))
    .wrap(Logger::new(r#""%r" %s %bB %Dms"#))     // registered LAST, so it runs FIRST
    .service(web::scope("/admin").wrap(from_fn(require_api_key)) …)   // only for /admin
```

- **Order:** each `.wrap` wraps everything registered before it, so the **last** `wrap` is the outermost: it sees the request first and the response last. (Axum's `ServiceBuilder` reads top to bottom instead; `Router::layer` called repeatedly is bottom-up like Actix.)
- **`middleware::from_fn(async_fn)`** is the easy way to write one (Axum has the same idea under the same name). It gets the `ServiceRequest` and `Next`; it may answer early (`req.into_response(…)`) or call `next.call(req).await`. Early answers and the handler's answer have different body types, joined with `map_into_left_body` / `map_into_right_body` (`EitherBody`).
- **`Logger`** writes one line per request through the **`log`** crate, so something must print `log` records: here `tracing-subscriber`, which forwards them (its `tracing-log` feature is on by default). Format codes: `%r` request line, `%s` status, `%b` bytes, `%D` milliseconds, `%{name}o` a response header.
- Writing a full `Transform`/`Service` pair by hand is how the built-in middleware are made; `from_fn` and `wrap_fn` cover most needs.

The real output of `middleware` (one request; `[…]` lines are the example's own prints):

```text
GET /admin/stats (key: None)
  [timer]     before /admin/stats
  [api key]   refused /admin/stats
  [timer]     after: 401 Unauthorized
2026-11-14T13:40:09.082276Z  INFO actix_web::middleware::logger: "GET /admin/stats HTTP/1.1" 401
  => 401 "missing or wrong x-api-key" (x-shop: true, x-took: true)
```

> 💡 **Analogy: layers of security at an airport.** Every passenger goes through the same checkpoints in order: the **logbook** at the entrance (`Logger`), the **stopwatch** (`wrap_fn`), and, only for the staff corridor, a **badge reader** (`from_fn` on a scope) that can turn people back before they reach the plane (the handler). On the way out, they pass the same checkpoints in reverse.

▶ `cargo run -p actix_web_basics --example middleware`

## 8. Testing: no network, then the real server

```rust
#[actix_web::test]                                 // like #[tokio::test]
async fn create_then_get_a_note() {
    let app = test::init_service(app(AppState::new())).await;          // the App as a Service
    let request = TestRequest::post().uri("/api/notes").set_json(json!({ "title": "x" }));
    let response = test::call_service(&app, request.to_request()).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let note: Note = test::read_body_json(response).await;
}
```

- `init_service` + `call_service` run the whole app (routing, extractors, middleware) with **no socket**: fast and deterministic. `call_and_read_body_json` does both in one go. This is Axum's `oneshot`.
- [`tests/e2e.rs`](tests/e2e.rs) binds `127.0.0.1:0`, starts the real server with `rt::spawn`, talks to it over TCP with the tiny client in [`src/client.rs`](src/client.rs), and stops it with `handle.stop(true)`: nothing is left running, and every client call has a 5-second timeout.
- Build the `App` in one function (`app()` or a `configure(cfg: &mut web::ServiceConfig)`) so `main`, the tests and the examples all use the same one.

> 💡 **Analogy: a flight simulator, then a test flight.** `init_service` is the simulator: every switch and dial works, but the plane never leaves the hangar. The end-to-end test is one short real flight, with an instructor (the timeout) ready to take over.

## 9. The actor heritage, and Actix's runtime

**Where the name comes from.** Actix began as an **actor framework**, the `actix` crate: actors are objects that own their state and talk only by sending each other messages, each on its own event loop (an **arbiter**). Early Actix-web was built on those actors. Today's Actix-web doesn't use them at all (its dependency list has `actix-rt`, `actix-server`, `actix-http`… but no `actix`), but it kept the runtime design. WebSocket handlers can still be actors, in separate crates.

**The runtime today.** `actix-rt` runs each worker as a **single-threaded Tokio runtime** (`new_current_thread` plus a `LocalSet`). A connection is accepted by one worker and its requests stay on that thread from start to finish. Consequences:

- **Handler futures don't have to be `Send`.** You may hold an `Rc` or a `RefCell` borrow across `.await` in an Actix handler; Axum (a work-stealing runtime) refuses that at compile time. The `runtime_and_actors` example keeps per-worker notes in an `Rc<RefCell<Vec<_>>>`:

```text
main runs on thread Some("main")
rt::spawn on this thread computed 42
GET /note/milk  -> actix-rt|system:0|arbiter:0 holds 1 note(s)
GET /note/eggs  -> actix-rt|system:0|arbiter:1 holds 1 note(s)
GET /note/bread -> actix-rt|system:0|arbiter:0 holds 2 note(s)
GET /note/jam   -> actix-rt|system:0|arbiter:1 holds 2 note(s)
GET /slow-sum/1000 -> sum 1..=1000 = 500500
```

- **No work stealing.** A worker stuck on one slow request can't hand its other connections to an idle worker. Blocking code (heavy computation, synchronous I/O) is even more harmful than in Axum: run it with **`web::block(|| …)`**, which uses a separate thread pool (Tokio's `spawn_blocking` in Axum).
- **`actix_web::rt::spawn`** is Tokio's `spawn_local`: the task stays on the current worker (and may hold `!Send` values). `tokio::spawn` still works inside Actix, but needs a `Send` future.
- `#[actix_web::main]` starts an Actix `System` on the main thread; Tokio libraries (`tokio::net`, `tokio::time`, `sqlx`…) work inside it, because underneath it is Tokio.

> 💡 **Analogy: one cook per kitchen.** In Axum's big shared kitchen, any free cook may finish any dish, so every tool must be safe to pass between cooks (`Send`). In Actix, the cook who starts a dish finishes it, so a cook can use their own **personal knife** (`Rc`) that never leaves their kitchen. The price: if one cook is stuck kneading dough (blocking work), nobody else can take over their orders, so heavy jobs go to the **back room** (`web::block`).

▶ `cargo run -p actix_web_basics --example runtime_and_actors`

## 10. The same API in Axum and in Actix

The notes API, side by side. Axum's version is in [lesson 06/01](../01_axum/src/routes.rs); Actix's is [`src/routes.rs`](src/routes.rs).

**Creating a note:**

```rust
// Axum
async fn create_note(
    State(state): State<AppState>,
    Extension(RequestId(request_id)): Extension<RequestId>,
    payload: Result<Json<NewNote>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let Json(new) = payload?;
    let new = new.validate().map_err(ApiError::Validation)?;
    let note = state.store.create(new);
    let location = format!("/api/notes/{}", note.id);
    Ok((StatusCode::CREATED, [(header::LOCATION, location)], Json(note)))
}
```

```rust
// Actix-web
#[post("/notes")]
async fn create_note(
    state: web::Data<AppState>,
    request_id: web::ReqData<RequestId>,
    body: web::Json<NewNote>,              // errors handled once, in JsonConfig
) -> Result<HttpResponse, ApiError> {
    let new = body.into_inner().validate().map_err(ApiError::Validation)?;
    let note = state.notes.create(new);
    Ok(HttpResponse::Created()
        .insert_header((header::LOCATION, format!("/api/notes/{}", note.id)))
        .json(note))
}
```

**Wiring and serving:**

```rust
// Axum
let app = Router::new()
    .route("/", get(hello))
    .nest("/api", Router::new()
        .route("/notes", get(list_notes).post(create_note))
        .route("/notes/{id}", get(get_note).patch(update_note).delete(delete_note)))
    .fallback(no_route)
    .with_state(state);
axum::serve(TcpListener::bind("127.0.0.1:3000").await?, app).await?;
```

```rust
// Actix-web
let state = AppState::new();                        // web::Data<AppState>
HttpServer::new(move || {
    App::new()
        .app_data(state.clone())
        .service(hello)
        .service(web::scope("/api")
            .service(list_notes).service(create_note)
            .service(get_note).service(update_note).service(delete_note))
        .default_service(web::to(no_route))
})
.bind(("127.0.0.1", 8080))?
.run()
.await?;
```

| Job | Axum | Actix-web |
|---|---|---|
| Error type → response | `impl IntoResponse for ApiError` | `impl ResponseError for ApiError` |
| Extractor errors in your format | take `Result<Json<T>, JsonRejection>` and convert | `JsonConfig::error_handler` (once, for every handler) |
| Request id into the handler | `Extension<RequestId>` | `web::ReqData<RequestId>` |
| Wrong method on a known path | 405 | 404 here (separate macros), 405 with one `web::resource` |
| Test without a network | `app.oneshot(request)` | `test::call_service(&app, request)` |
| Graceful shutdown | `.with_graceful_shutdown(signal)` | built in for Ctrl+C; `handle.stop(true)` |

**Which should you pick?** Both are production-ready and fast; the difference is taste and ecosystem. Axum shares the Tower middleware ecosystem with hyper, tonic (gRPC) and many others, checks state at compile time and has a smaller API. Actix-web is batteries-included (compression, cookies, sessions, static files and WebSockets in its own crates), runs `!Send` handlers, and has years of production use. This course's capstone uses Axum.

> 💡 **Analogy: two car makers.** Both cars drive you to school. One has its controls arranged one way, the other another way, and they're built in differently organised factories. Once you can drive one, the other takes an afternoon: the steering wheel (handlers), pedals (extractors) and mirrors (middleware) are all still there.

## The mini project: the notes API again

| File | What's in it |
|---|---|
| [`src/notes.rs`](src/notes.rs) | `Note`, `NewNote`, `UpdateNote`, `ListParams`, validation that reports every problem, `NoteStore` (an `RwLock<BTreeMap>`): no Actix in it |
| [`src/error.rs`](src/error.rs) | `ApiError` with `ResponseError`: JSON bodies, and `rejected()` for extractor errors |
| [`src/routes.rs`](src/routes.rs) | `AppState`, the `#[get]`/`#[post]`/`#[patch]`/`#[delete]` handlers, `configure` (extractor configs, `/api` scope, default service), `app()` (request-id `wrap_fn` + `Logger`) |
| [`src/lib.rs`](src/lib.rs) | `serve` (workers, 5-second shutdown timeout), `init_logging` |
| [`src/client.rs`](src/client.rs) | a tiny raw-TCP HTTP client for examples and tests |
| [`src/main.rs`](src/main.rs) | listens on `127.0.0.1:8080` (or the port you pass) |
| [`tests/api.rs`](tests/api.rs) | 11 tests through `init_service` / `call_service` |
| [`tests/e2e.rs`](tests/e2e.rs) | the real server on port 0, two workers, graceful stop |

| Request | Answer |
|---|---|
| `GET /` | `200 Hello from Actix!` |
| `GET /health` | `200 {"notes":N,"status":"ok"}` |
| `GET /api/notes?tag=rust&done=false&limit=10&offset=0` | `200 [ ...notes... ]` |
| `POST /api/notes` `{"title":"..","body":"..","tags":[..]}` | `201` + `location: /api/notes/1` + the note; `422` with `details` if invalid |
| `GET /api/notes/{id}` | `200` the note, `404 {"error":"note 7 not found"}`, `400` for a non-number |
| `PATCH /api/notes/{id}` `{"done":true}` | `200` the changed note |
| `DELETE /api/notes/{id}` | `204`, then `404` |
| anything else | `404 {"error":"no route for PUT /api/notes/1"}` |

A real session (`curl -i`, abridged):

```text
$ curl -i -X POST http://127.0.0.1:8080/api/notes -H "content-type: application/json" -d '{"title":"Learn actix","tags":["rust"]}'
HTTP/1.1 201 Created
content-length: 69
x-request-id: req-1
content-type: application/json
location: /api/notes/1

{"id":1,"title":"Learn actix","body":"","tags":["rust"],"done":false}

$ curl -i -X POST http://127.0.0.1:8080/api/notes -H "content-type: application/json" -d '{"title":"","tags":["Bad Tag"]}'
HTTP/1.1 422 Unprocessable Entity

{"details":["title must not be empty","tag \"Bad Tag\" must be 1 to 20 characters of a-z, 0-9 or -"],"error":"validation failed"}
```

And the server's log:

```text
INFO actix_server::builder: starting 2 workers
INFO actix_server::server: starting service: "actix-web-service-127.0.0.1:8080", workers: 2, listening on: 127.0.0.1:8080
INFO actix_web_basics::routes: created note id=1 request_id=1
INFO actix_web::middleware::logger: "POST /api/notes HTTP/1.1" 201 69B 1.010200ms id=req-1
```

> 💡 **Analogy:** the same little post office for notes as in the Axum lesson, rebuilt by a different builder: the counters, forms and complaints desk all do the same jobs, but each sorting room now has its own clerk, and they all share one filing cabinet.

## Common mistakes

| You wrote | What happens | Usually the fix |
|---|---|---|
| A handler returning a plain struct | E0277 the trait bound `Book: Responder` is not satisfied | `web::Json(book)` |
| `Result<_, MyError>` without `impl ResponseError` | E0277 the trait bound `MyError: ResponseError` is not satisfied | `impl ResponseError for MyError` (needs `Debug + Display`) |
| `HttpServer::new(\|\| … state.clone() …)` | E0373 closure may outlive the current function | `move`, with a clone made before the closure |
| `web::Data::new(...)` inside the factory for shared data | every worker gets its own copy | Create it once outside, clone it in |
| `web::Data<T>` in a handler, no `.app_data(...)` | 500 at run time, not a compile error | Register the data; test every route |
| `.app_data(state)` where `state: T` but the handler wants `web::Data<T>` | 500: the types must match exactly | `.app_data(web::Data::new(state))` |
| A `#[get]` handler registered with `.route(..., web::get().to(handler))` | E0277 the trait bound `handler: Handler<_>` is not satisfied | `.service(handler)`: the macro turned the fn into a service |
| Forgot `.service(handler)` for a macro handler | 404 | Register every handler |
| Expected 405 for a wrong method | 404 with separate route macros | One `web::resource` with several `.route`s |
| Expected Axum's statuses | Actix's own: `Path` 404, `Json` 400 | Set `PathConfig`/`JsonConfig` error handlers |
| JSON POST without `content-type: application/json` | 400 Content type error | Send the header (or `JsonConfig::content_type_required(false)`) |
| `srv.call(req)` in `wrap_fn` | no method named `call` | `use actix_web::dev::Service as _;` |
| Middleware order surprises | the last `wrap` runs first | Register the outermost one last |
| `std::thread::sleep` or heavy work in a handler | that worker's other connections stall | `web::block(…)` |
| No log lines from `Logger` | `log` records go nowhere | Install a logger (`tracing-subscriber`, `env_logger`) |
| `compress-zstd` (a default feature) on a machine without a C compiler | `zstd-sys` fails to build | `default-features = false` plus what you need |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p actix_web_basics --example hello_server
cargo run -p actix_web_basics --example routing
cargo run -p actix_web_basics --example extractors
cargo run -p actix_web_basics --example shared_state
cargo run -p actix_web_basics --example responders_and_errors
cargo run -p actix_web_basics --example middleware
cargo run -p actix_web_basics --example runtime_and_actors
```

**2. Do the exercises** in `exercises/`:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_not_a_responder` | Make it compile (E0277): return JSON | `cargo run -p actix_web_basics --features exercises --bin ex01_fix_not_a_responder` |
| 2 | `ex02_fix_error_not_response_error` | Make it compile (E0277): `impl ResponseError` | `cargo run -p actix_web_basics --features exercises --bin ex02_fix_error_not_response_error` |
| 3 | `ex03_fix_factory_borrows_state` | Make it compile (E0373, then E0382): share state across workers | `cargo run -p actix_web_basics --features exercises --bin ex03_fix_factory_borrows_state` |
| 4 | `ex04_bookmark_handlers` | Three handlers: 200/201/404/422 and `Location`; pass 4 tests | `cargo test -p actix_web_basics --features exercises --bin ex04_bookmark_handlers` |
| 5 | `ex05_json_errors` | `ResponseError` + a `JsonConfig` error handler; pass 4 tests | `cargo test -p actix_web_basics --features exercises --bin ex05_json_errors` |
| 6 | `ex06_api_key_middleware` | A `from_fn` middleware on a scope; pass 4 tests | `cargo test -p actix_web_basics --features exercises --bin ex06_api_key_middleware` |

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p actix_web_basics --features solutions --bins`).

**4. The mini project:** run it (`cargo run -p actix_web_basics`), try the `curl` commands above, stop it with Ctrl+C, then:

- Open [lesson 06/01's `src/routes.rs`](../01_axum/src/routes.rs) next to this lesson's and match every handler, extractor and middleware.
- Add `GET /api/notes/count?tag=rust` returning `{"count": N}`.
- Turn the five `/api` macros into one `web::resource("/notes/{id}")` with three `.route`s, and watch `PUT /api/notes/1` become a 405 (then update the test).

```bash
cargo test -p actix_web_basics   # 6 unit + 11 API + 1 end-to-end tests
```

## Checklist

- [ ] `HttpServer` and `App`; the factory runs once per worker; `bind`/`listen`, `run`, `ServerHandle::stop`
- [ ] Route macros, `route`, `web::resource`, `web::scope`, guards, `default_service`; when 404 and when 405
- [ ] Extractors: `web::Path`, `web::Query`, `web::Json`, `web::Form`, `HttpRequest`, `web::ReqData`; `JsonConfig`/`PathConfig`/`QueryConfig`
- [ ] Shared state with `web::Data` (an `Arc`), and the per-worker trap
- [ ] Responders: `&str`, `web::Json`, `HttpResponse`, `customize`, `Either`, your own `impl Responder`
- [ ] Errors: `ResponseError` (`status_code`, `error_response`), `actix_web::Error`, the `Error*` helpers
- [ ] Middleware: `Logger`, `wrap_fn`, `from_fn`, `DefaultHeaders`; order; on a scope
- [ ] Testing with `actix_web::test` (`init_service`, `call_service`), and the real server on port 0
- [ ] The actor heritage; one single-threaded runtime per worker; `!Send` handlers; `web::block`; `rt::spawn`
- [ ] Reading the same API in Axum and in Actix
- [ ] Exercises 1–6 done without looking at the solutions

## Recap

- `HttpServer` runs an `App` on several workers, calling your factory closure **once per worker**; `ServerHandle::stop(true)` (or Ctrl+C) shuts down gracefully.
- Routes come from macros (`#[get("/x")]`) or builders (`route`, `resource`, `scope`), with guards and a default service.
- Extractors work like Axum's; their limits and error formats are set with `JsonConfig`, `PathConfig` and `QueryConfig`.
- `web::Data<T>` is an `Arc`: create it once outside the factory and clone it in.
- Handlers return a `Responder`; errors implement `ResponseError`.
- Middleware: `Logger`, `wrap_fn`, `from_fn`; the last `wrap` is the outermost.
- `actix_web::test` tests the app with no network; a port-0 server tests it for real.
- Each worker is a single-threaded runtime: handlers may hold `!Send` values, and blocking work goes to `web::block`.

**In one sentence for a child:** Actix-web is a restaurant with several little kitchens, each with one cook who finishes every dish they start, all following the same recipe book and sharing one till.

**Next:** [06 Tauri and Bevy](../06_tauri_and_bevy/README.md): desktop apps and games, as optional later tracks.
