//! Exercise 2: make it compile (E0277): an error type Actix can answer with.
//!
//! `borrow` returns `Result<String, LibraryError>`. For a `Result` to be a
//! responder, Actix must be able to turn the error into a response, which
//! means the error type implements `actix_web::ResponseError`.
//!
//! Implement `ResponseError for LibraryError` (it already has `Debug` and
//! `Display`, which `ResponseError` requires):
//!   - `status_code`: 404 for `NoSuchBook`, 409 for `AlreadyBorrowed`;
//!   - `error_response`: a JSON body `{"error": "<the Display text>"}`
//!     (`HttpResponse::build(self.status_code()).json(...)`).
//!
//! When it works, `main` prints 200, 404 and 409 responses.
//!
//! Run: cargo run -p actix_web_basics --features exercises --bin ex02_fix_error_not_response_error

use std::fmt;

use actix_web::test::{self, TestRequest};
use actix_web::{App, get, web};

#[derive(Debug)]
enum LibraryError {
    NoSuchBook(u32),
    AlreadyBorrowed(u32),
}

impl fmt::Display for LibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LibraryError::NoSuchBook(id) => write!(f, "there is no book {id}"),
            LibraryError::AlreadyBorrowed(id) => write!(f, "book {id} is already borrowed"),
        }
    }
}

#[get("/borrow/{id}")]
async fn borrow(id: web::Path<u32>) -> Result<String, LibraryError> {
    match id.into_inner() {
        1 => Ok("enjoy book 1".to_string()),
        2 => Err(LibraryError::AlreadyBorrowed(2)),
        other => Err(LibraryError::NoSuchBook(other)),
    }
}

#[actix_web::main]
async fn main() {
    let app = test::init_service(App::new().service(borrow)).await;
    for id in [1, 2, 3] {
        let request = TestRequest::get()
            .uri(&format!("/borrow/{id}"))
            .to_request();
        let response = test::call_service(&app, request).await;
        let status = response.status();
        let body = test::read_body(response).await;
        println!("book {id}: {status} {}", String::from_utf8_lossy(&body));
    }
}
