//! Exercise 1: make it compile (E0277): a handler must return a `Responder`.
//!
//! `book` returns a plain struct. Actix doesn't know how to turn a `Book`
//! into an HTTP response: should it be JSON? HTML? Some other format?
//! Read the error, then fix the handler so it answers with JSON. Change the
//! return type and the last line only; `Book` itself already derives
//! `Serialize`.
//!
//! (Axum reports this mistake as E0277 too, phrased as "the trait `Handler`
//! is not implemented", and the fix is the same: wrap the value in `Json`.)
//!
//! Run: cargo run -p actix_web_basics --features exercises --bin ex01_fix_not_a_responder

use actix_web::test::{self, TestRequest};
use actix_web::{App, get, web};
use serde::Serialize;

#[derive(Serialize)]
struct Book {
    title: String,
    pages: u32,
}

#[get("/book")]
async fn book() -> Book {
    Book {
        title: "Treasure Island".to_string(),
        pages: 292,
    }
}

#[actix_web::main]
async fn main() {
    let app = test::init_service(App::new().service(book)).await;
    let response = test::call_service(&app, TestRequest::get().uri("/book").to_request()).await;
    println!("status: {}", response.status());
    let body = test::read_body(response).await;
    println!("body:   {}", String::from_utf8_lossy(&body));
    let _ = web::Json(()); // `web` is used once you've fixed it
}
