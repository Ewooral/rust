//! The HTTP side: shared state, handlers, the router and its middleware.
//!
//! | Method and path         | Handler       | Success                     |
//! |-------------------------|---------------|-----------------------------|
//! | `GET /`                 | `hello`       | 200 `Hello from Axum!`      |
//! | `GET /health`           | `health`      | 200 `{"status":"ok",...}`   |
//! | `GET /api/notes`        | `list_notes`  | 200 JSON array              |
//! | `POST /api/notes`       | `create_note` | 201 + `Location` + the note |
//! | `GET /api/notes/{id}`   | `get_note`    | 200 the note                |
//! | `PATCH /api/notes/{id}` | `update_note` | 200 the changed note        |
//! | `DELETE /api/notes/{id}`| `delete_note` | 204 no body                 |

use crate::error::ApiError;
use crate::notes::{ListParams, NewNote, Note, NoteStore, UpdateNote};
use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::extract::{DefaultBodyLimit, Path, Query, Request, State};
use axum::http::{HeaderName, HeaderValue, Method, StatusCode, Uri, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Extension, Json, Router};
use serde_json::{Value, json};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

/// Everything the handlers share. `Clone` is required by axum: every
/// request gets its own copy, which is cheap because it's just an `Arc`.
#[derive(Clone, Default)]
pub struct AppState {
    pub store: Arc<NoteStore>,
}

/// Requests that take longer than this get `408 Request Timeout`.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
/// Bodies larger than this get `413 Payload Too Large` (axum's default is 2 MB).
pub const BODY_LIMIT: usize = 64 * 1024;

/// The whole application. Tests call this and send it requests with
/// `tower::ServiceExt::oneshot`, without opening a socket.
pub fn app(state: AppState) -> Router {
    let notes = Router::new()
        .route("/notes", get(list_notes).post(create_note))
        .route(
            "/notes/{id}",
            get(get_note).patch(update_note).delete(delete_note),
        );

    // ServiceBuilder applies layers top to bottom: the first one listed is
    // the OUTERMOST, so it sees the request first and the response last.
    let middleware = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(request_id))
        .layer(cors())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            REQUEST_TIMEOUT,
        ));

    Router::new()
        .route("/", get(hello))
        .route("/health", get(health))
        .nest("/api", notes)
        .fallback(no_route)
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
        .layer(middleware)
        .with_state(state)
}

/// Lets JavaScript on any web page call this API from a browser. A real
/// API would list its own front-end's origin instead of `Any`.
fn cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE])
        .max_age(Duration::from_secs(3600))
}

// ----- middleware ----------------------------------------------------------

pub static X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

/// The id of the current request, available to handlers as
/// `Extension<RequestId>`.
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

/// A hand-written middleware: reuse the client's `x-request-id` header or
/// make one up, store it in the request's extensions for the handlers,
/// call the rest of the app, and copy the id onto the response.
async fn request_id(mut request: Request, next: Next) -> Response {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let id = match request.headers().get(&X_REQUEST_ID) {
        Some(value) => value.to_str().unwrap_or("invalid").to_string(),
        None => format!("req-{}", COUNTER.fetch_add(1, Ordering::Relaxed)),
    };
    request.extensions_mut().insert(RequestId(id.clone()));

    let mut response = next.run(request).await; // <- the handler runs here

    if let Ok(value) = HeaderValue::from_str(&id) {
        response.headers_mut().insert(X_REQUEST_ID.clone(), value);
    }
    response
}

// ----- handlers ------------------------------------------------------------

/// The route this folder started with: `GET /` on port 3000.
async fn hello() -> &'static str {
    "Hello from Axum!"
}

async fn health(State(state): State<AppState>) -> Json<Value> {
    Json(json!({ "status": "ok", "notes": state.store.len() }))
}

/// `GET /api/notes?tag=rust&done=false&limit=10&offset=0`
///
/// Wrapping an extractor in `Result` means it never rejects the request on
/// its own: we get the rejection and turn it into our JSON error with `?`.
async fn list_notes(
    State(state): State<AppState>,
    query: Result<Query<ListParams>, QueryRejection>,
) -> Result<Json<Vec<Note>>, ApiError> {
    let Query(params) = query?;
    Ok(Json(state.store.list(&params)))
}

/// `POST /api/notes` with a JSON body. The body extractor must be the LAST
/// argument, because it consumes the request.
async fn create_note(
    State(state): State<AppState>,
    Extension(RequestId(request_id)): Extension<RequestId>,
    payload: Result<Json<NewNote>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let Json(new) = payload?;
    let new = new.validate().map_err(ApiError::Validation)?;
    let note = state.store.create(new);
    tracing::info!(id = note.id, %request_id, "created note");
    let location = format!("/api/notes/{}", note.id);
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, location)],
        Json(note),
    ))
}

async fn get_note(
    State(state): State<AppState>,
    id: Result<Path<u64>, PathRejection>,
) -> Result<Json<Note>, ApiError> {
    let Path(id) = id?;
    state.store.get(id).map(Json).ok_or(ApiError::NotFound(id))
}

async fn update_note(
    State(state): State<AppState>,
    id: Result<Path<u64>, PathRejection>,
    payload: Result<Json<UpdateNote>, JsonRejection>,
) -> Result<Json<Note>, ApiError> {
    let Path(id) = id?;
    let Json(change) = payload?;
    let change = change.validate().map_err(ApiError::Validation)?;
    state
        .store
        .update(id, change)
        .map(Json)
        .ok_or(ApiError::NotFound(id))
}

async fn delete_note(
    State(state): State<AppState>,
    id: Result<Path<u64>, PathRejection>,
) -> Result<StatusCode, ApiError> {
    let Path(id) = id?;
    if state.store.delete(id) {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound(id))
    }
}

/// Runs when no route matches: a JSON 404 instead of an empty one.
async fn no_route(method: Method, uri: Uri) -> ApiError {
    ApiError::NoRoute(format!("{method} {uri}"))
}
