<!-- hand-written lesson: gen_course must not overwrite -->
# Axum

**Docs:** [axum on docs.rs](https://docs.rs/axum/0.8) · [axum examples](https://github.com/tokio-rs/axum/tree/main/examples) · [Tokio tutorial](https://tokio.rs/tokio/tutorial)
· Building blocks: [`tower`](https://docs.rs/tower) (`Service`, `Layer`, `ServiceExt::oneshot`), [`tower-http`](https://docs.rs/tower-http) (ready-made middleware), [`hyper`](https://docs.rs/hyper) (HTTP), [`serde`](https://serde.rs) (JSON)
· Logging: [`tracing`](https://docs.rs/tracing), [`tracing-subscriber`](https://docs.rs/tracing-subscriber)

Run it: `cargo run -p axum_basics`, then open <http://127.0.0.1:3000>  ·  Test it: `cargo test -p axum_basics`

In [04/04](../../04_systems_level/04_web_server_project/README.md) you built a web server by hand: parse the request line, pick a page, write `Content-Length`, manage a thread pool. **Axum** does all of that for you, on top of async I/O ([04/02](../../04_systems_level/02_async_await/README.md)), and leaves you to write only the interesting part: what each URL should do. After this lesson you can:

- route requests to handlers by method and path, including captures like `/notes/{id}` and nested routers;
- pull exactly what a handler needs out of a request with **extractors** (`Path`, `Query`, `Json`, `State`, headers) and understand the errors when one refuses;
- return anything that implements **`IntoResponse`**, including your own error type;
- share state safely between requests, add **middleware** (logging, timeouts, CORS, your own), shut down gracefully;
- **test** handlers without a network, and end to end on a real socket.

This lesson uses axum 0.8 (0.8.9 in `Cargo.lock`), which runs on hyper 1 and tower 0.5.

> 💡 **The big picture: a post office.** Letters (requests) arrive at the door. A **sorting rack** (the `Router`) reads the address on each envelope, a method and a path like `POST /api/notes`, and drops it into the right pigeonhole. Behind each pigeonhole sits a **clerk** (a handler). The clerk never opens envelopes: **helpers** (extractors) do that and hand over exactly what the clerk asked for, the house number (`Path`), the notes on the back (`Query`), the letter itself (`Json`). The clerk writes a **reply** (a response). All clerks share one **filing cabinet** (state), and every letter goes through the **security scanner and the date stamp** at the door (middleware). The building's **runners** who carry everything around are Tokio; the people who speak the postal language are hyper; and **Tower** is the rule that every tray in the building is the same shape, so any machine can be slotted between any two others.

## 1. What axum is, and the first program

Axum is a thin layer that connects three crates:

| Crate | Job | You touch it when… |
|---|---|---|
| **Tokio** | the async runtime: tasks, timers, sockets | `#[tokio::main]`, `TcpListener::bind` |
| **hyper** | speaks HTTP/1 and HTTP/2 on a socket | almost never directly |
| **Tower** | the `Service` trait: "a thing that turns a request into a future response", plus `Layer`s that wrap services | middleware and testing |
| **axum** | routing, extractors, responses, glue | all the time |

This folder started with this program (now [`examples/hello.rs`](examples/hello.rs), changed only so it stops by itself):

```rust
use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(|| async { "Hello from Axum!" }));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
```

- `Router::new().route(path, method_router)` registers a route; `get(handler)` says "for GET requests".
- The handler is an `async` closure returning `&'static str`, which axum turns into a `200 OK` with `content-type: text/plain; charset=utf-8`.
- `axum::serve(listener, app)` accepts connections forever, giving each one its own Tokio task. Compare that with the 150 lines of thread pool in 04/04.

```text
listening on http://127.0.0.1:9252
GET / -> 200 "Hello from Axum!"
server stopped
```

> 💡 **Analogy: opening the post office.** `bind` puts the building at a street address (the port). `Router` is the empty sorting rack, and `.route("/", get(...))` adds one pigeonhole labelled "GET /" with a clerk who always replies "Hello". `serve` unlocks the front door and keeps it open.

▶ `cargo run -p axum_basics --example hello`

## 2. Routing and handlers

```rust
fn users() -> Router {
    Router::new()
        .route("/", get(list_users).post(create_user))   // two methods, one path
        .route("/{id}", get(show_user))                   // capture one segment
}

let app = Router::new()
    .route("/", get(root))
    .route("/files/{*path}", get(show_file))              // capture the rest, slashes too
    .nest("/users", users())
    .fallback(|| async { (StatusCode::NOT_FOUND, "nothing here") });
```

A **handler** is an `async fn` whose arguments are **extractors** and whose return type implements **`IntoResponse`**. That's the whole contract; there is no trait to implement and no base class.

What the router answers (real output of the example):

```text
GET    /                      -> 200 OK "home"
GET    /users                 -> 200 OK "all users"
POST   /users                 -> 201 Created "user created"
GET    /users/42              -> 200 OK "user 42"
GET    /users/abc             -> 400 Bad Request "Invalid URL: Cannot parse `abc` to a `u32`"
GET    /files/docs/notes.txt  -> 200 OK "file \"docs/notes.txt\""
POST   /ping                  -> 200 OK "pong"
GET    /ping                  -> 405 Method Not Allowed ""  (allow: POST)
DELETE /users/42              -> 405 Method Not Allowed ""  (allow: GET,HEAD)
GET    /nope                  -> 404 Not Found "nothing here"
```

- Right path, wrong method gives **405** with an `allow` header, not 404. A `get` route answers `HEAD` too.
- **axum 0.8 changed the capture syntax** from `/:id` to `/{id}`. Old tutorials still show `:id`, and it isn't a compile error but a panic when the router is built:

```text
Path segments must not start with `:`. For capture groups, use `{capture}`. If you meant to literally match a segment starting with a colon, call `without_v07_checks` on the router.
```

> 💡 **Analogy: pigeonholes with labels.** Each pigeonhole is labelled with a method and a path. `{id}` is a hole that accepts any house number on that street; `{*path}` accepts the whole rest of the address. A letter for an address with no hole goes to the **"return to sender"** tray (the fallback, 404). A letter for the right address but the wrong kind of service, like a parcel at the stamps counter, is turned away with a note saying which counter to use (405 and `allow`).

▶ `cargo run -p axum_basics --example routing`

## 3. Extractors: unpacking the request

```rust
async fn comment(
    State(config): State<Config>,     // shared data from .with_state(...)
    Path(post_id): Path<u32>,         // /posts/{id}/comments
    headers: HeaderMap,               // every header
    Json(text): Json<String>,         // the body, parsed as JSON: LAST
) -> String { … }
```

| Extractor | Reads | Example |
|---|---|---|
| `Path<T>` | captures from the path | `Path(id): Path<u64>`, `Path((owner, repo)): Path<(String, String)>` |
| `Query<T>` | `?a=1&b=2`, into a serde struct | `Query(p): Query<ListParams>` |
| `Json<T>` | the body as JSON (needs `Content-Type: application/json`) | `Json(new): Json<NewNote>` |
| `HeaderMap` | all headers | `headers.get(header::USER_AGENT)` |
| `State<S>` | the router's shared state | `State(state): State<AppState>` |
| `Method`, `Uri`, `Extension<T>`, `String`, `Bytes`, `Form<T>` | the method, the URI, data set by middleware, the raw body… | |

`Path(id): Path<u64>` in an argument list is a **pattern**: it unwraps the tuple struct right there, so `id` is a plain `u64` in the body.

**The one ordering rule:** the request body can only be read once, so at most one extractor may consume it (`Json`, `String`, `Bytes`, `Form`, `Request`), and it must be the **last** argument. The others only read the head and can come in any order. Break the rule and the error is famously unhelpful. The real error from exercise 1 (abridged):

```text
error[E0277]: the trait bound `fn(Json<String>, State<_>, Path<u32>) -> _ {add_comment}: Handler<_, _>` is not satisfied
   --> 06_framework_mastery\01_axum\exercises\ex01_fix_extractor_order.rs:42:45
    |
 42 |         .route("/posts/{id}/comments", post(add_comment))
    |                                        ---- ^^^^^^^^^^^ the trait `Handler<_, _>` is not implemented for fn item `fn(Json<String>, State<AppState>, Path<u32>) -> _ {add_comment}`
    |                                        |
    |                                        required by a bound introduced by this call
    |
    = note: Consider using `#[axum::debug_handler]` to improve the error message
```

Take the hint. **`#[axum::debug_handler]`** (the `macros` feature) checks the handler itself. With it on the line above `async fn add_comment` (so the handler moves down one line), the first error becomes (this one has no error code):

```text
error: `Json<_>` consumes the request body and thus must be the last argument to the handler function
  --> 06_framework_mastery\01_axum\exercises\ex01_fix_extractor_order.rs:33:17
   |
33 |     Json(text): Json<String>,
   |                 ^^^^
```

**Rejections.** When an extractor can't make sense of the request, the handler never runs and axum answers by itself (real output):

```text
GET /search?page=2                         -> 400 Bad Request Failed to deserialize query string: missing field `q`
POST /sign-up   (no Content-Type)          -> 415 Unsupported Media Type Expected request with `Content-Type: application/json`
POST /sign-up   {"name":"Ada"              -> 400 Bad Request Failed to parse the request body as JSON: EOF while parsing an object at line 1 column 13
POST /sign-up   {"name":"Ada"}             -> 422 Unprocessable Entity Failed to deserialize the JSON body into the target type: missing field `age` at line 1 column 14
POST /sign-up   {"name":"Ada","age":300}   -> 422 Unprocessable Entity Failed to deserialize the JSON body into the target type: age: invalid value: integer `300`, expected u8 at line 1 column 23
```

Those replies are plain text. To answer with **your** JSON error instead, wrap the extractor in `Result`: it then never rejects, and you decide what to do (the mini project does this):

```rust
async fn create_note(payload: Result<Json<NewNote>, JsonRejection>) -> Result<…, ApiError> {
    let Json(new) = payload?;   // From<JsonRejection> for ApiError makes `?` work
```

> 💡 **Analogy: the helpers who open the post.** The clerk lists what they want: "the house number, the notes on the back, and the letter". A helper fetches each one. Most helpers only **read the envelope**, so any number of them can look, in any order. But one helper **opens** the envelope and takes the letter out (the body), and after that there's nothing left to open, so that helper always goes last. If a helper can't do their job (the letter is in French, the house number is "abc"), the letter goes straight back with a note, and the clerk never sees it (a rejection).

▶ `cargo run -p axum_basics --example extractors`

## 4. Responses and `IntoResponse`

Anything that implements `IntoResponse` can be returned. The common ones, with what they set (real output of the example):

| Return | Status | `content-type` |
|---|---|---|
| `&'static str`, `String` | 200 | `text/plain; charset=utf-8` |
| `StatusCode::NO_CONTENT` | 204 | none (empty body) |
| `Json(value)` | 200 | `application/json` |
| `Html("<h1>Hello</h1>")` | 200 | `text/html; charset=utf-8` |
| `(StatusCode::CREATED, [(header::LOCATION, "/items/7")], Json(..))` | 201 | from the last part, plus the extra headers |
| `Redirect::permanent("/text")` | 308 | `location: /text` |
| `Result<T, E>` | whichever of `T` or `E` it holds | both must implement `IntoResponse` |
| `Response::builder().status(..).header(..).body(..)` | anything | anything: the low-level way |

A **tuple** is applied left to right: an optional status first, then any headers (arrays of pairs, or a `HeaderMap`), then the body last. Returning `impl IntoResponse` hides the exact type, but it is still **one** type, so two branches that build different responses don't compile (verified with a scratch file):

```text
error[E0308]: `if` and `else` have incompatible types
 7 | |         Json(serde_json::json!({ "ok": true }))
   | |         --------------------------------------- expected because of this
 9 | |         (StatusCode::BAD_REQUEST, "flag was false")
   | |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `Json<Value>`, found `(StatusCode, &str)`
```

Ignore the compiler's suggestion to return `Box<dyn IntoResponse>`. Call `.into_response()` on each branch and return `Response`, or return a `Result`.

> 💡 **Analogy: the reply envelope.** Whatever the clerk hands back, a sticky note, a printed form or a parcel, the **packing desk** (`IntoResponse`) knows how to put it in a proper reply envelope with the right stamp (status) and the right label (`content-type`). A tuple is "stamp it *Created*, write *find it at /items/7* on the back, and put this form inside".

▶ `cargo run -p axum_basics --example responses`

## 5. JSON with serde

`Json<T>` uses [serde](https://serde.rs): `T: Deserialize` to read a body, `T: Serialize` to write one. The mini project's shapes:

```rust
#[derive(Serialize, Deserialize)]
pub struct Note { pub id: u64, pub title: String, pub body: String, pub tags: Vec<String>, pub done: bool }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]        // {"title":"x","colour":"red"} is a 422, not silently ignored
pub struct NewNote {
    pub title: String,               // required
    #[serde(default)] pub body: String,        // may be left out: ""
    #[serde(default)] pub tags: Vec<String>,   // may be left out: []
}

#[derive(Deserialize)]
pub struct UpdateNote { pub title: Option<String>, pub done: Option<bool>, … }  // PATCH: only what's present
```

- An `Option<T>` field may be missing or `null`; `#[serde(default)]` fills in `Default::default()`.
- **Parsing is not validation.** serde checks the *shape* (a string where a string belongs); your own code checks the *rules* (the title isn't blank, at most 5 tags). The mini project reports every broken rule at once with `422` and a `details` list.
- `serde_json::json!({ "error": msg })` builds a one-off JSON value without declaring a struct.

> 💡 **Analogy: a form with boxes.** `Deserialize` is reading a filled-in form: each box must hold the right kind of thing (a number in the age box). `deny_unknown_fields` rejects forms with extra scribbles in the margin. `#[serde(default)]` is "leave blank if not applicable". Validation is the clerk then checking that what's written *makes sense*: a name that is only spaces is a valid string, but not a valid name.

## 6. Shared state

```rust
#[derive(Clone)]
struct AppState {
    hits: Arc<AtomicU64>,                      // a counter: an atomic is enough
    votes: Arc<Mutex<HashMap<String, u32>>>,   // a map: needs a lock
    config: Arc<Config>,                       // read-only: no lock at all
}

let app = Router::new().route("/hit", get(hit)).with_state(state);

async fn hit(State(state): State<AppState>) -> String { … }
```

- axum gives every request **its own clone** of the state, so `S: Clone` is required. The real error from exercise 3 (abridged):

```text
error[E0277]: the trait bound `AppState: Clone` is not satisfied
   --> 06_framework_mastery\01_axum\exercises\ex03_fix_state_not_clone.rs:44:15
    |
 44 |     let app = Router::new()
    |               ^^^^^^^^^^^^^ the trait `Clone` is not implemented for `AppState`
    |
help: consider annotating `AppState` with `#[derive(Clone)]`
```

- `#[derive(Clone)]` alone compiles, but a clone of a `Vec` is a **copy**: each request would get its own list. Put the data behind an **`Arc`**, so a clone is one more pointer to the **same** data. 100 concurrent requests to `/hit` in the example leave the counter at exactly 100.
- Data that changes needs a lock or an atomic. A **std `Mutex`/`RwLock`** is right as long as the guard is dropped before any `.await`; `tokio::sync::Mutex` is only for holding a lock *while* waiting.
- Holding a std guard across `.await` makes the handler's future non-`Send`, and the error is again the vague "not a `Handler`". With `#[axum::debug_handler]` you get the real reason (exercise 2, abridged; no error code):

```text
error: future cannot be sent between threads safely
  --> 06_framework_mastery\01_axum\exercises\ex02_fix_guard_across_await.rs:42:1
   |
42 | #[axum::debug_handler]
   | ^^^^^^^^^^^^^^^^^^^^^^ future returned by `record_visit` is not `Send`
   |
   = help: within `impl Future<Output = String>`, the trait `Send` is not implemented for `std::sync::MutexGuard<'_, u64>`
note: future is not `Send` as this value is used across an await
  --> 06_framework_mastery\01_axum\exercises\ex02_fix_guard_across_await.rs:46:19
   |
44 |     let mut visits = state.visits.lock().unwrap();
   |         ---------- has type `std::sync::MutexGuard<'_, u64>` which is not `Send`
45 |     *visits += 1;
46 |     save(*visits).await;
   |                   ^^^^^ await occurs here, with `mut visits` maybe used later
```

- **`FromRef`** lets a handler extract just one piece, `State<Arc<Config>>`, from a bigger state. Another way to share data is `Extension<T>`, which middleware can set per request (the mini project's `RequestId`).

> 💡 **Analogy: the shared filing cabinet.** Every clerk gets a **key** to the same cabinet (a clone of an `Arc`), not a photocopy of the whole cabinet. A drawer that people write in has a **"one at a time" sign** (the lock). The rule that matters: never wander off to wait for the kettle while still holding the drawer open, because everyone else is stuck until you come back. Read-only drawers need no sign at all.

▶ `cargo run -p axum_basics --example shared_state`

## 7. Error handling with your own error type

Write one error enum, implement `IntoResponse` for it once, and let every handler return `Result<T, AppError>`:

```rust
enum AppError { NotFound(String), BadInput(String), Internal(String) }

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(what) => (StatusCode::NOT_FOUND, format!("{what} not found")),
            AppError::BadInput(why) => (StatusCode::BAD_REQUEST, why),
            AppError::Internal(details) => {
                eprintln!("internal error: {details}");          // the log gets the details
                (StatusCode::INTERNAL_SERVER_ERROR, "something went wrong".to_string())
            }
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

impl From<ParseIntError> for AppError { … }   // so `raw.parse()?` works
impl From<std::io::Error> for AppError { … }  // so `fs::read_to_string(..).await?` works
```

Real output:

```text
GET /square/12             -> 200 OK 12 squared is 144
GET /square/twelve         -> 400 Bad Request {"error":"not a number: invalid digit found in string"}
GET /users/2               -> 404 Not Found {"error":"user 2 not found"}
    [log] internal error: The system cannot find the path specified. (os error 3)
GET /report                -> 500 Internal Server Error {"error":"something went wrong"}
```

- **4xx** means the *client* did something wrong: say exactly what. **5xx** means *we* did: log the details, and send a generic message, because details (file paths, SQL, stack traces) help attackers more than users.
- Handlers never `unwrap()` user input: a panic in a handler doesn't crash an axum server (hyper drops that connection), but the client gets no answer at all.
- The mini project's `ApiError` also has `From<JsonRejection>`, `From<PathRejection>` and `From<QueryRejection>`, so even a bad request body comes back as `{"error": "..."}` JSON.

> 💡 **Analogy: the complaints desk.** Every problem goes to one desk that knows how to word the reply. If *you* filled the form in wrong, the desk tells you exactly which box. If *the office* lost your parcel, it says "sorry, something went wrong on our side" and writes the embarrassing details in the internal logbook, not on your reply.

▶ `cargo run -p axum_basics --example error_handling`

## 8. Middleware with Tower and tower-http

A Tower **`Service`** is "something that takes a request and returns a future of a response"; a router is one, and so is each handler. A **`Layer`** wraps a service in another service, which is all middleware is. That's why any Tower middleware works with axum, and why `tower-http` exists: a shelf of ready-made layers.

**Your own middleware** is an async fn with `axum::middleware::from_fn`:

```rust
async fn request_id(mut request: Request, next: Next) -> Response {
    // before: look at or change the request
    let mut response = next.run(request).await;      // the rest of the app runs here
    // after: look at or change the response
    response
}
```

**Order** (real output): with `ServiceBuilder`, the first layer listed is the outermost.

```text
  outer: before
    inner: before
      handler runs
    inner: after
  outer: after (200 OK)
```

Calling `Router::layer` one at a time is the other way round: each call wraps everything added so far. It also wraps only the routes added **before** it: a route added after the `.layer(..)` call doesn't get that middleware (verified: `/before` got the header, `/after` didn't). `route_layer` runs only when a route matched, so an unknown path stays a 404 instead of, say, a 401 (exercise 6).

**tower-http** layers used in this lesson:

| Layer | Does | Real output |
|---|---|---|
| `TraceLayer::new_for_http()` | one log span per request: method, URI, status, latency | `DEBUG request{method=GET uri=/ version=HTTP/1.1}: finished processing request latency=0 ms status=200` |
| `TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, dur)` | answers 408 if the inner service takes too long, and drops (cancels) it | `-> 408 Request Timeout after about 50 ms` |
| `CorsLayer` | answers browsers' cross-origin checks | `access-control-allow-origin: http://localhost:5173` |
| `DefaultBodyLimit::max(n)` (axum's own) | rejects bigger bodies with 413; the default is 2 MB | |

`TimeoutLayer::new(dur)` still exists but has been deprecated since tower-http 0.6.7 in favour of `with_status_code`, which makes you choose the status. The logs need a **subscriber** to go anywhere: `tracing_subscriber::fmt().with_env_filter(...)`, and `RUST_LOG=tower_http=debug` picks how much (`init_tracing` in `src/lib.rs`).

**CORS, explained.** Browsers enforce the **same-origin policy**: JavaScript on a page from `http://localhost:5173` may *send* a request to `http://127.0.0.1:3000` (a different origin: scheme, host or port differ), but may not *read* the response unless the response says `access-control-allow-origin` for that origin. For anything beyond a simple GET or form post (a JSON body, `PATCH`, `DELETE`, custom headers) the browser first sends a **preflight**: an `OPTIONS` request asking "may I?", and `CorsLayer` answers it:

```text
  preflight -> 200 OK
    access-control-allow-methods: GET,POST
    access-control-allow-headers: content-type
    access-control-allow-origin: http://localhost:5173
```

CORS only matters for browsers; `curl`, servers and tests ignore it. It is not authentication: it protects users' browsers, not your API.

> 💡 **Analogy: the doors every letter passes through.** Middleware is a row of doors between the street and the sorting rack: a **date-stamp machine** (request id), a **logbook** at the entrance (`TraceLayer`), a **kitchen timer** that gives up on a clerk who takes too long (`TimeoutLayer`), and a **scale** that refuses parcels over 64 KB (`DefaultBodyLimit`). Every letter goes in through all the doors and the reply comes back out through the same doors in reverse order. **CORS** is the building's rule about **notes passed from the neighbours' house**: the browser (a careful parent) won't let the child read a reply from another house unless that house wrote "it's fine for your house to read this" on it, and for anything unusual the parent phones first to ask (the preflight).

▶ `cargo run -p axum_basics --example middleware`

## 9. Nested and merged routers

```rust
let notes = Router::new()
    .route("/notes", get(list_notes).post(create_note))
    .route("/notes/{id}", get(get_note).patch(update_note).delete(delete_note));

Router::new()
    .route("/", get(hello))
    .nest("/api", notes)          // /api/notes, /api/notes/{id}
    .merge(other_router())        // routes added as they are
    .fallback(no_route)
    .with_state(state)
```

- `nest(prefix, router)` puts a router under a prefix: build each part of the API in its own function or module. `merge` combines routers side by side.
- `Router<S>` means "a router still **missing** a state of type `S`". Routes and nested routers can use `State<AppState>`; one `.with_state(state)` at the top fills it in for all of them, after which the type is `Router<()>`, ready to serve.
- A version prefix (`/api/v1`) is just another `nest`.

> 💡 **Analogy: departments.** A big post office has a **parcels department** with its own little sorting rack. `nest("/api", notes)` hangs that whole rack behind a door marked "/api". `merge` pushes two racks together into one wide rack. `with_state` hands every department the key to the same filing cabinet in one go.

## 10. Graceful shutdown

```rust
axum::serve(listener, app)
    .with_graceful_shutdown(async { tokio::signal::ctrl_c().await.ok(); })
    .await
```

When the future you pass completes, axum stops accepting connections and waits for the requests in progress. The example uses a oneshot channel in place of Ctrl+C (real output, Windows):

```text
[  0 ms] client: GET / -> 200 "quick"
[100 ms] main: "Ctrl+C" while /slow is still running
[100 ms] server: shutdown signal, no new connections
[250 ms] client: new connection not accepted (no answer in 100 ms)
[400 ms] client: GET /slow -> 200 "slow, but finished"
[400 ms] main: serve() returned: every connection is closed
```

(On Linux the new connection is refused at once; Windows keeps retrying a refused connect for about two seconds, so the example gives up after 100 ms.)

> 💡 **Analogy: closing time, again.** Same idea as the ice-cream shop in 04/04, but axum does the work: turn the sign to CLOSED (stop accepting), let the customers already inside finish, then lock up. In 04/04 you had to ring your own doorbell to wake the door-minder; here `with_graceful_shutdown` handles it.

▶ `cargo run -p axum_basics --example graceful_shutdown`

## 11. Testing: without a network, then end to end

**Without a network.** A `Router` is a Tower `Service`, so `tower::ServiceExt::oneshot` can hand it one request and give back the response. No port, no socket, no waiting:

```rust
use tower::ServiceExt; // for `oneshot`

let app = app(AppState::default());
let response = app
    .oneshot(Request::get("/api/notes/99").body(Body::empty()).unwrap())
    .await
    .unwrap();
assert_eq!(response.status(), StatusCode::NOT_FOUND);
let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
```

`oneshot` consumes the service, so clone the router (cheap) for each request. Because state is just `Arc`s, a test can keep a clone and check what the handlers did: `assert_eq!(state.store.len(), 2)`.

**End to end.** One test in `tests/e2e.rs` runs the real server: bind `127.0.0.1:0`, `tokio::spawn` the server with a oneshot shutdown signal, talk to it over TCP, send the signal, and check that `serve` returns within 5 seconds. The client ([`src/client.rs`](src/client.rs)) is 60 lines on a raw `TcpStream`, because HTTP is still the text you wrote by hand in 04/04.

> 💡 **Analogy: practising with toy letters.** `oneshot` is slipping a letter straight into the sorting rack from inside the building: you test every pigeonhole and clerk without the street, the door or the postman. The end-to-end test is posting one real letter through the real front door, to make sure the door is actually connected to the rack.

## The mini project: a notes JSON API

| File | What's in it |
|---|---|
| [`src/notes.rs`](src/notes.rs) | `Note`, `NewNote`, `UpdateNote`, `ListParams` (serde), validation that reports every problem, `NoteStore` (an `RwLock<BTreeMap>`) |
| [`src/error.rs`](src/error.rs) | `ApiError`: `IntoResponse` with JSON bodies, `From` for the three rejections |
| [`src/routes.rs`](src/routes.rs) | `AppState`, the handlers, `app()`: nested `/api` router, fallback, request-id middleware, `TraceLayer`, `CorsLayer`, `TimeoutLayer`, `DefaultBodyLimit` |
| [`src/lib.rs`](src/lib.rs) | `serve` (with graceful shutdown), `init_tracing` |
| [`src/client.rs`](src/client.rs) | the tiny raw-TCP HTTP client for examples and tests |
| [`src/main.rs`](src/main.rs) | listens on `127.0.0.1:3000` (as the original program did), stops on Ctrl+C |
| [`tests/api.rs`](tests/api.rs) | 13 tests through `oneshot` |
| [`tests/e2e.rs`](tests/e2e.rs) | 1 test over real TCP on port 0 |

| Request | Answer |
|---|---|
| `GET /` | `200 Hello from Axum!` (the original route) |
| `GET /health` | `200 {"notes":N,"status":"ok"}` |
| `GET /api/notes?tag=rust&done=false&limit=10&offset=0` | `200 [ ...notes... ]` |
| `POST /api/notes` `{"title":"..","body":"..","tags":[..]}` | `201` + `location: /api/notes/1` + the note; `422` with `details` if invalid |
| `GET /api/notes/{id}` | `200` the note, `404 {"error":"note 7 not found"}` |
| `PATCH /api/notes/{id}` `{"done":true}` | `200` the changed note |
| `DELETE /api/notes/{id}` | `204`, then `404` |
| anything else | `404 {"error":"no route for GET /nope"}`, or `405` for a wrong method |

A real session (`curl -i`, abridged):

```text
$ curl -i -X POST http://127.0.0.1:3000/api/notes -H "content-type: application/json" -d '{"title":"Learn axum","tags":["rust"]}'
HTTP/1.1 201 Created
content-type: application/json
location: /api/notes/1
access-control-allow-origin: *
x-request-id: req-2

{"id":1,"title":"Learn axum","body":"","tags":["rust"],"done":false}

$ curl -i -X POST http://127.0.0.1:3000/api/notes -H "content-type: application/json" -d '{"title":"","tags":["Bad Tag"]}'
HTTP/1.1 422 Unprocessable Entity

{"details":["title must not be empty","tag \"Bad Tag\" must be 1 to 20 characters of a-z, 0-9 or -"],"error":"validation failed"}
```

And the server's log for the first request (the `INFO` line is the handler's own `tracing::info!`):

```text
DEBUG request{method=POST uri=/api/notes version=HTTP/1.1}: tower_http::trace::on_request: started processing request
 INFO request{method=POST uri=/api/notes version=HTTP/1.1}: axum_basics::routes: created note id=1 request_id=req-2
DEBUG request{method=POST uri=/api/notes version=HTTP/1.1}: tower_http::trace::on_response: finished processing request latency=1 ms status=201
```

> 💡 **Analogy:** you've built a small post office for notes: a sorting rack with an `/api` department, helpers who open every envelope carefully, one complaints desk that words every refusal the same way, a shared filing cabinet with a lock, a date stamp and a logbook at the door, and a closing routine that lets everyone inside finish.

## Common mistakes

| You wrote | Error | Usually the fix |
|---|---|---|
| `Json(..)` (or `String`, `Bytes`, `Form`) before other extractors | E0277 `…: Handler<_, _>` is not satisfied | Body extractor **last**; add `#[axum::debug_handler]` to see why |
| A std `MutexGuard` (or `Rc`) alive across `.await` in a handler | E0277 not a `Handler`; with `debug_handler`: future cannot be sent between threads safely | Drop the guard first (a block), or `tokio::sync::Mutex` |
| State struct without `#[derive(Clone)]` | E0277 the trait bound `AppState: Clone` is not satisfied | `#[derive(Clone)]`, with the data behind `Arc` |
| `#[derive(Clone)]` on state holding a `Vec`/`HashMap` directly | compiles; every request sees its own copy, changes vanish | `Arc<Mutex<..>>` / `Arc<RwLock<..>>` / `Arc<T>` |
| `.route("/users/:id", ..)` (axum 0.7 syntax) | panic: Path segments must not start with `:` | `/users/{id}` |
| Different response types in `if`/`else` | E0308 `if` and `else` have incompatible types | `.into_response()` on each branch, return `Response` |
| Expected a JSON error but got plain text | 400/415/422 from a rejection | `Result<Json<T>, JsonRejection>` + `From<JsonRejection>` for your error |
| Sent JSON without `Content-Type: application/json` | 415 Unsupported Media Type | Set the header (`curl -H "content-type: application/json"`) |
| Added a route after `.layer(..)` | the middleware doesn't run for it | Add `.layer` after all routes, or use `ServiceBuilder` once |
| `TimeoutLayer::new(..)` | deprecation warning | `TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, ..)` |
| Sent internal error details to the client | leaks paths/SQL/secrets | Log the details, return a generic 500 |
| No `tracing_subscriber` initialised | `TraceLayer` and `tracing::info!` print nothing | `tracing_subscriber::fmt().with_env_filter(..).init()` |
| Expected CORS to protect the API | `curl` and servers ignore CORS | Use real authentication (exercise 6 shows the middleware shape) |
| Bound `127.0.0.1:3000` in a test | `AddrInUse` when the server is already running | Bind `127.0.0.1:0` and read `local_addr()` |
| `app.oneshot(..)` twice | E0382 use of moved value | `app.clone().oneshot(..)` |

## Practise

**1. Run the examples**, in order:

```bash
cargo run -p axum_basics --example hello
cargo run -p axum_basics --example routing
cargo run -p axum_basics --example extractors
cargo run -p axum_basics --example responses
cargo run -p axum_basics --example shared_state
cargo run -p axum_basics --example error_handling
cargo run -p axum_basics --example middleware
cargo run -p axum_basics --example graceful_shutdown
```

**2. Do the exercises** in `exercises/`:

| # | Exercise | Kind | Command |
|---|---|---|---|
| 1 | `ex01_fix_extractor_order` | Make it compile (E0277), meet `#[axum::debug_handler]` | `cargo run -p axum_basics --features exercises --bin ex01_fix_extractor_order` |
| 2 | `ex02_fix_guard_across_await` | Make it compile (E0277; the cause has no code) | `cargo run -p axum_basics --features exercises --bin ex02_fix_guard_across_await` |
| 3 | `ex03_fix_state_not_clone` | Make it compile (E0277 `AppState: Clone`), then share with `Arc` | `cargo run -p axum_basics --features exercises --bin ex03_fix_state_not_clone` |
| 4 | `ex04_todo_handlers` | Router + `Path`, `Query`, `Json`, `State` handlers, pass 3 tests | `cargo test -p axum_basics --features exercises --bin ex04_todo_handlers` |
| 5 | `ex05_api_error` | `IntoResponse` for an error enum, `From<JsonRejection>`, pass 3 tests | `cargo test -p axum_basics --features exercises --bin ex05_api_error` |
| 6 | `ex06_api_key_middleware` | `from_fn_with_state` middleware: 401/403/pass through, pass 3 tests | `cargo test -p axum_basics --features exercises --bin ex06_api_key_middleware` |

Exercises 4 to 6 test through `oneshot`, exactly as section 11 describes.

**3. Check yourself** only after a real attempt: reference answers are in `solutions/` (`cargo test -p axum_basics --features solutions --bins`, or `cargo run -p axum_basics --features solutions --bin sol01_fix_extractor_order` and so on).

**4. Run the mini project** and poke it:

```bash
cargo run -p axum_basics                     # http://127.0.0.1:3000
curl -i http://127.0.0.1:3000/
curl -i -X POST http://127.0.0.1:3000/api/notes -H "content-type: application/json" -d '{"title":"Learn axum","tags":["rust"]}'
curl -i "http://127.0.0.1:3000/api/notes?tag=rust"
curl -i -X PATCH http://127.0.0.1:3000/api/notes/1 -H "content-type: application/json" -d '{"done":true}'
curl -i -X DELETE http://127.0.0.1:3000/api/notes/1
cargo test -p axum_basics                    # 9 unit + 13 oneshot + 1 end-to-end tests
```

(In PowerShell, call `curl.exe` rather than `curl`; if the JSON quoting gives trouble, run these from Git Bash.) Then try: add a `GET /api/tags` route that lists every tag in use, with a test in `tests/api.rs`.

## Checklist

- [ ] Axum = routing + extractors + responses on top of Tokio, hyper and Tower; `axum::serve`
- [ ] Routes, methods, `{id}` and `{*rest}` captures, 404 vs 405, `fallback`
- [ ] Extractors: `Path`, `Query`, `Json`, `HeaderMap`, `State`; body extractor last; rejections; `Result<Extractor, Rejection>`
- [ ] `#[axum::debug_handler]` for readable handler errors
- [ ] Responses: `IntoResponse`, tuples, `Json`, `Html`, `Redirect`, `.into_response()`
- [ ] JSON with serde: derives, `#[serde(default)]`, `deny_unknown_fields`, validation after parsing
- [ ] Shared state: `Clone` + `Arc`, locks and atomics, no std guard across `.await`, `FromRef`, `Extension`
- [ ] A custom error type implementing `IntoResponse`, with `From` impls for `?`
- [ ] Middleware: `from_fn`, `ServiceBuilder` order, `route_layer`, `TraceLayer`, `TimeoutLayer`, `CorsLayer`, `DefaultBodyLimit`
- [ ] Nested and merged routers; `Router<S>` and `with_state`
- [ ] Graceful shutdown with `with_graceful_shutdown`
- [ ] Testing with `tower::ServiceExt::oneshot`, and one end-to-end test on `127.0.0.1:0`
- [ ] Exercises 1–6 and the notes API done without looking at the solutions

## Recap

- An axum app is a `Router` of routes; each route maps a method and a path (`/notes/{id}`) to a **handler**: an async fn taking **extractors** and returning something that implements **`IntoResponse`**.
- Extractors read the request; only one may read the body, and it goes last. A refused extractor answers by itself (400/415/422) unless you take `Result<Extractor, Rejection>`.
- State is cloned per request: derive `Clone`, share with `Arc`, protect changes with a lock or an atomic, and never hold a std guard across `.await`.
- One error enum with `IntoResponse` and `From` impls lets handlers use `?` and keeps error responses consistent; log 5xx details, don't send them.
- Middleware is Tower layers: your own with `from_fn`, or `tower-http`'s tracing, timeouts and CORS; mind the order.
- `with_graceful_shutdown` finishes requests in progress; `oneshot` tests handlers without a network, and one port-0 test proves the real server works.

**In one sentence for a child:** axum is a post office where a sorting rack sends every letter to the right clerk, helpers open the envelopes for them, and every reply goes back out through the same front door.

**Next:** [02 Tokio](../02_tokio/README.md): the async runtime underneath axum, in depth.
