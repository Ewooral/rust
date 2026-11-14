//! Solution to exercise 4: write the handlers of a small bookmarks API.
//!
//! The state, the routes and the tests are done. Implement the three
//! handlers (each `todo!()`):
//!
//!   GET  /bookmarks          -> 200, JSON array of every bookmark, in id order
//!   POST /bookmarks          -> body {"url": "...", "title": "..."}
//!                               201 + `Location: /bookmarks/{id}` + the new bookmark as JSON;
//!                               422 + {"error": "url must start with http:// or https://"}
//!                               if the url doesn't start with one of those
//!   GET  /bookmarks/{id}     -> 200 + the bookmark, or 404 + {"error": "no bookmark {id}"}
//!
//! Ids start at 1. Use `web::Data<AppState>` for the state, `web::Json<NewBookmark>`
//! for the body, `web::Path<u32>` for the id, and `HttpResponse::Created()`,
//! `HttpResponse::NotFound()`, `HttpResponse::UnprocessableEntity()` with
//! `.insert_header(...)` and `.json(...)`.
//!
//! Test: cargo test -p actix_web_basics --features solutions --bin sol04_bookmark_handlers

use std::sync::Mutex;

use actix_web::http::header;
use actix_web::{App, HttpResponse, get, post, web};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Bookmark {
    id: u32,
    url: String,
    title: String,
}

#[derive(Debug, Deserialize)]
struct NewBookmark {
    url: String,
    title: String,
}

#[derive(Default)]
struct AppState {
    bookmarks: Mutex<Vec<Bookmark>>,
}

#[get("/bookmarks")]
async fn list(state: web::Data<AppState>) -> HttpResponse {
    let bookmarks = state.bookmarks.lock().unwrap();
    HttpResponse::Ok().json(&*bookmarks)
}

#[post("/bookmarks")]
async fn create(state: web::Data<AppState>, body: web::Json<NewBookmark>) -> HttpResponse {
    let NewBookmark { url, title } = body.into_inner();
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return HttpResponse::UnprocessableEntity()
            .json(json!({ "error": "url must start with http:// or https://" }));
    }
    let mut bookmarks = state.bookmarks.lock().unwrap();
    let id = bookmarks.len() as u32 + 1;
    let bookmark = Bookmark { id, url, title };
    bookmarks.push(bookmark.clone());
    HttpResponse::Created()
        .insert_header((header::LOCATION, format!("/bookmarks/{id}")))
        .json(bookmark)
}

#[get("/bookmarks/{id}")]
async fn get_one(state: web::Data<AppState>, id: web::Path<u32>) -> HttpResponse {
    let id = id.into_inner();
    let bookmarks = state.bookmarks.lock().unwrap();
    match bookmarks.iter().find(|b| b.id == id) {
        Some(bookmark) => HttpResponse::Ok().json(bookmark),
        None => HttpResponse::NotFound().json(json!({ "error": format!("no bookmark {id}") })),
    }
}

fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(list).service(create).service(get_one);
}

#[actix_web::main]
async fn main() {
    use actix_web::test::{self, TestRequest};
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(AppState::default()))
            .configure(configure),
    )
    .await;
    let request = TestRequest::post()
        .uri("/bookmarks")
        .set_json(json!({ "url": "https://www.rust-lang.org", "title": "Rust" }));
    let response = test::call_service(&app, request.to_request()).await;
    println!("POST /bookmarks -> {}", response.status());
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;
    use actix_web::test::{self, TestRequest};
    use serde_json::Value;

    macro_rules! app {
        () => {
            test::init_service(
                App::new()
                    .app_data(web::Data::new(AppState::default()))
                    .configure(configure),
            )
            .await
        };
    }

    fn new_bookmark(url: &str, title: &str) -> TestRequest {
        TestRequest::post()
            .uri("/bookmarks")
            .set_json(json!({ "url": url, "title": title }))
    }

    #[actix_web::test]
    async fn creates_with_201_and_a_location() {
        let app = app!();
        let response = test::call_service(
            &app,
            new_bookmark("https://doc.rust-lang.org", "Docs").to_request(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            "/bookmarks/1"
        );
        let created: Bookmark = test::read_body_json(response).await;
        assert_eq!(
            created,
            Bookmark {
                id: 1,
                url: "https://doc.rust-lang.org".into(),
                title: "Docs".into()
            }
        );
    }

    #[actix_web::test]
    async fn lists_in_order_and_gets_one() {
        let app = app!();
        for (url, title) in [("https://a.example", "A"), ("http://b.example", "B")] {
            test::call_service(&app, new_bookmark(url, title).to_request()).await;
        }
        let all: Vec<Bookmark> =
            test::call_and_read_body_json(&app, TestRequest::get().uri("/bookmarks").to_request())
                .await;
        let titles: Vec<&str> = all.iter().map(|b| b.title.as_str()).collect();
        assert_eq!(titles, ["A", "B"]);
        let second: Bookmark = test::call_and_read_body_json(
            &app,
            TestRequest::get().uri("/bookmarks/2").to_request(),
        )
        .await;
        assert_eq!(second.url, "http://b.example");
    }

    #[actix_web::test]
    async fn unknown_ids_are_404_json() {
        let app = app!();
        let response =
            test::call_service(&app, TestRequest::get().uri("/bookmarks/9").to_request()).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body: Value = test::read_body_json(response).await;
        assert_eq!(body, json!({ "error": "no bookmark 9" }));
    }

    #[actix_web::test]
    async fn bad_urls_are_422() {
        let app = app!();
        let response =
            test::call_service(&app, new_bookmark("ftp://old.example", "Old").to_request()).await;
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body: Value = test::read_body_json(response).await;
        assert_eq!(
            body,
            json!({ "error": "url must start with http:// or https://" })
        );
        let all: Vec<Bookmark> =
            test::call_and_read_body_json(&app, TestRequest::get().uri("/bookmarks").to_request())
                .await;
        assert!(all.is_empty());
    }
}
