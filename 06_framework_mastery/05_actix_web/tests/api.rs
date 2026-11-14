//! The API, tested without a network: `actix_web::test::init_service` builds
//! the app as a `Service`, and `call_service` hands it a fake request.
//! (In Axum the same job is done by `tower::ServiceExt::oneshot`.)

use actix_web::http::{Method, StatusCode};
use actix_web::test::{self, TestRequest};
use actix_web_basics::{AppState, Note, app};
use serde_json::{Value, json};

/// Builds the app and returns a closure-like helper: send a request, get the
/// status, a header and the JSON body back.
macro_rules! call {
    ($app:expr, $req:expr) => {{
        let response = test::call_service(&$app, $req.to_request()).await;
        let status = response.status();
        let request_id = response
            .headers()
            .get("x-request-id")
            .map(|v| v.to_str().unwrap().to_string());
        let body = test::read_body(response).await;
        let json: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
        (status, json, request_id, body)
    }};
}

fn post_note(body: Value) -> TestRequest {
    TestRequest::post().uri("/api/notes").set_json(body)
}

#[actix_web::test]
async fn hello_and_health() {
    let app = test::init_service(app(AppState::new())).await;
    let (status, _, _, body) = call!(app, TestRequest::get().uri("/"));
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "Hello from Actix!");
    let (status, json, _, _) = call!(app, TestRequest::get().uri("/health"));
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json, json!({ "status": "ok", "notes": 0 }));
}

#[actix_web::test]
async fn create_then_get_a_note() {
    let app = test::init_service(app(AppState::new())).await;
    let request = post_note(json!({ "title": "  Learn actix ", "tags": ["rust"] }));
    let response = test::call_service(&app, request.to_request()).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(response.headers().get("location").unwrap(), "/api/notes/1");
    let note: Note = test::read_body_json(response).await;
    assert_eq!(note.title, "Learn actix");
    assert!(!note.done);

    let fetched: Note =
        test::call_and_read_body_json(&app, TestRequest::get().uri("/api/notes/1").to_request())
            .await;
    assert_eq!(fetched, note);
}

#[actix_web::test]
async fn validation_errors_are_422_with_every_problem() {
    let app = test::init_service(app(AppState::new())).await;
    let (status, json, _, _) = call!(app, post_note(json!({ "title": "", "tags": ["Bad Tag"] })));
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(json["error"], "validation failed");
    assert_eq!(json["details"].as_array().unwrap().len(), 2);
}

#[actix_web::test]
async fn bad_json_is_400_in_our_format() {
    let app = test::init_service(app(AppState::new())).await;
    let unknown_field = post_note(json!({ "title": "x", "colour": "red" }));
    let (status, json, _, _) = call!(app, unknown_field);
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .contains("unknown field `colour`")
    );

    let not_json = TestRequest::post()
        .uri("/api/notes")
        .insert_header(("content-type", "text/plain"))
        .set_payload("hello");
    let (status, json, _, _) = call!(app, not_json);
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(json["error"], "Content type error");
}

#[actix_web::test]
async fn too_big_a_body_is_413() {
    let app = test::init_service(app(AppState::new())).await;
    let huge = "x".repeat(20_000);
    let (status, json, _, _) = call!(app, post_note(json!({ "title": "t", "body": huge })));
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .contains("larger than allowed")
    );
}

#[actix_web::test]
async fn missing_notes_and_bad_ids() {
    let app = test::init_service(app(AppState::new())).await;
    let (status, json, _, _) = call!(app, TestRequest::get().uri("/api/notes/7"));
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(json, json!({ "error": "note 7 not found" }));

    let (status, json, _, _) = call!(app, TestRequest::get().uri("/api/notes/seven"));
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .starts_with("Path deserialize error")
    );
}

#[actix_web::test]
async fn list_filters_and_pages() {
    let app = test::init_service(app(AppState::new())).await;
    for (title, tags) in [("a", vec!["rust"]), ("b", vec![]), ("c", vec!["rust"])] {
        call!(app, post_note(json!({ "title": title, "tags": tags })));
    }
    let titles = |json: Value| -> Vec<String> {
        json.as_array()
            .unwrap()
            .iter()
            .map(|n| n["title"].as_str().unwrap().to_string())
            .collect()
    };
    let (_, json, _, _) = call!(app, TestRequest::get().uri("/api/notes?tag=rust"));
    assert_eq!(titles(json), ["a", "c"]);
    let (_, json, _, _) = call!(app, TestRequest::get().uri("/api/notes?limit=1&offset=1"));
    assert_eq!(titles(json), ["b"]);
    let (status, json, _, _) = call!(app, TestRequest::get().uri("/api/notes?colour=red"));
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .contains("unknown field `colour`")
    );
}

#[actix_web::test]
async fn patch_and_delete() {
    let app = test::init_service(app(AppState::new())).await;
    call!(app, post_note(json!({ "title": "a" })));
    let patch = TestRequest::patch()
        .uri("/api/notes/1")
        .set_json(json!({ "done": true }));
    let (status, json, _, _) = call!(app, patch);
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["done"], true);

    let (status, _, _, body) = call!(app, TestRequest::delete().uri("/api/notes/1"));
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(body.is_empty());
    let (status, _, _, _) = call!(app, TestRequest::delete().uri("/api/notes/1"));
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn unknown_routes_and_methods_get_a_json_404() {
    let app = test::init_service(app(AppState::new())).await;
    let (status, json, _, _) = call!(app, TestRequest::get().uri("/nope"));
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(json, json!({ "error": "no route for GET /nope" }));
    // Axum answers a known path with the wrong method with 405. Each
    // `#[get]`/`#[put]` macro here is its own resource with a method guard,
    // so a PUT matches none of them and falls through to the default service.
    let put = TestRequest::default()
        .method(Method::PUT)
        .uri("/api/notes/1");
    let (status, json, _, _) = call!(app, put);
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(json, json!({ "error": "no route for PUT /api/notes/1" }));
}

#[actix_web::test]
async fn every_response_gets_a_request_id() {
    let app = test::init_service(app(AppState::new())).await;
    let (_, _, first, _) = call!(app, TestRequest::get().uri("/"));
    let (_, _, second, _) = call!(app, TestRequest::get().uri("/nope"));
    assert_eq!(first.as_deref(), Some("req-1"));
    assert_eq!(second.as_deref(), Some("req-2"));
}

#[actix_web::test]
async fn state_is_shared_between_app_instances() {
    // HttpServer builds one App per worker; they share the SAME Data.
    let state = AppState::new();
    let worker_a = test::init_service(app(state.clone())).await;
    let worker_b = test::init_service(app(state.clone())).await;
    call!(worker_a, post_note(json!({ "title": "made on worker a" })));
    let (_, json, _, _) = call!(worker_b, TestRequest::get().uri("/api/notes/1"));
    assert_eq!(json["title"], "made on worker a");
    assert_eq!(state.notes.len(), 1);
}
