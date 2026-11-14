//! Shared state, the handlers, and how they are wired into an `App`.

use std::sync::atomic::{AtomicU64, Ordering};

use actix_web::body::MessageBody;
use actix_web::dev::{Service as _, ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::http::header::{self, HeaderName, HeaderValue};
use actix_web::middleware::Logger;
use actix_web::{App, HttpMessage, HttpRequest, HttpResponse, delete, get, patch, post, web};
use serde_json::{Value, json};

use crate::error::ApiError;
use crate::notes::{ListParams, NewNote, Note, NoteStore, UpdateNote};

/// Everything the handlers share. It lives in ONE `web::Data<AppState>`
/// (an `Arc` inside), made once before the server starts, so every worker
/// thread sees the same notes.
#[derive(Debug, Default)]
pub struct AppState {
    /// The notes.
    pub notes: NoteStore,
    requests: AtomicU64,
}

impl AppState {
    /// Fresh state, wrapped in `web::Data` ready for `App::app_data`.
    pub fn new() -> web::Data<AppState> {
        web::Data::new(AppState::default())
    }

    fn next_request_id(&self) -> u64 {
        self.requests.fetch_add(1, Ordering::Relaxed) + 1
    }
}

/// The id the request-id middleware gives each request. Handlers can take it
/// as `web::ReqData<RequestId>` (it lives in the request's extensions).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestId(pub u64);

/// `GET /`
#[get("/")]
async fn hello() -> &'static str {
    "Hello from Actix!"
}

/// `GET /health`
#[get("/health")]
async fn health(state: web::Data<AppState>) -> web::Json<Value> {
    web::Json(json!({ "status": "ok", "notes": state.notes.len() }))
}

/// `GET /api/notes?tag=rust&done=false&limit=10&offset=0`
#[get("/notes")]
async fn list_notes(
    state: web::Data<AppState>,
    params: web::Query<ListParams>,
) -> web::Json<Vec<Note>> {
    web::Json(state.notes.list(&params))
}

/// `POST /api/notes` with a JSON body: `201 Created` and a `Location` header.
#[post("/notes")]
async fn create_note(
    state: web::Data<AppState>,
    request_id: web::ReqData<RequestId>,
    body: web::Json<NewNote>,
) -> Result<HttpResponse, ApiError> {
    let new = body.into_inner().validate().map_err(ApiError::Validation)?;
    let note = state.notes.create(new);
    tracing::info!(id = note.id, request_id = request_id.0, "created note");
    Ok(HttpResponse::Created()
        .insert_header((header::LOCATION, format!("/api/notes/{}", note.id)))
        .json(note))
}

/// `GET /api/notes/{id}`
#[get("/notes/{id}")]
async fn get_note(
    state: web::Data<AppState>,
    id: web::Path<u64>,
) -> Result<web::Json<Note>, ApiError> {
    let id = id.into_inner();
    state
        .notes
        .get(id)
        .map(web::Json)
        .ok_or(ApiError::NotFound(id))
}

/// `PATCH /api/notes/{id}` with the fields to change.
#[patch("/notes/{id}")]
async fn update_note(
    state: web::Data<AppState>,
    id: web::Path<u64>,
    body: web::Json<UpdateNote>,
) -> Result<web::Json<Note>, ApiError> {
    let id = id.into_inner();
    let change = body.into_inner().validate().map_err(ApiError::Validation)?;
    state
        .notes
        .update(id, change)
        .map(web::Json)
        .ok_or(ApiError::NotFound(id))
}

/// `DELETE /api/notes/{id}`: `204 No Content`.
#[delete("/notes/{id}")]
async fn delete_note(
    state: web::Data<AppState>,
    id: web::Path<u64>,
) -> Result<HttpResponse, ApiError> {
    let id = id.into_inner();
    if state.notes.delete(id) {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(ApiError::NotFound(id))
    }
}

/// Anything no route matched (Actix's default would be an empty 404).
async fn no_route(req: HttpRequest) -> Result<HttpResponse, ApiError> {
    Err(ApiError::NoRoute(format!(
        "{} {}",
        req.method(),
        req.path()
    )))
}

/// Registers the routes and the extractor settings. A `configure` function
/// keeps `App` building in one place, and lets tests reuse it.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        // Extractor settings: limits, and errors in OUR JSON format.
        .app_data(
            web::JsonConfig::default()
                .limit(16 * 1024)
                .error_handler(|err, _req| ApiError::rejected(&err).into()),
        )
        .app_data(
            web::PathConfig::default().error_handler(|err, _req| ApiError::rejected(&err).into()),
        )
        .app_data(
            web::QueryConfig::default().error_handler(|err, _req| ApiError::rejected(&err).into()),
        )
        .service(hello)
        .service(health)
        .service(
            web::scope("/api")
                .service(list_notes)
                .service(create_note)
                .service(get_note)
                .service(update_note)
                .service(delete_note),
        )
        .default_service(web::to(no_route));
}

/// The whole application: state, middleware and routes.
///
/// `HttpServer::new` calls a factory like this **once per worker thread**,
/// so `state` must be created outside and cloned in (cloning `web::Data`
/// clones an `Arc`: every worker shares the same store).
pub fn app(
    state: web::Data<AppState>,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse<impl MessageBody>,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new()
        .app_data(state)
        // Middleware registered LAST runs FIRST: Logger sees every request
        // on the way in, and the final response (with x-request-id) on the
        // way out.
        .wrap_fn(|req, srv| {
            let id = req
                .app_data::<web::Data<AppState>>()
                .map_or(0, |state| state.next_request_id());
            req.extensions_mut().insert(RequestId(id));
            let response = srv.call(req);
            async move {
                let mut response = response.await?;
                if let Ok(value) = HeaderValue::from_str(&format!("req-{id}")) {
                    response
                        .headers_mut()
                        .insert(HeaderName::from_static("x-request-id"), value);
                }
                Ok(response)
            }
        })
        .wrap(Logger::new(r#""%r" %s %bB %Dms id=%{x-request-id}o"#))
        .configure(configure)
}
