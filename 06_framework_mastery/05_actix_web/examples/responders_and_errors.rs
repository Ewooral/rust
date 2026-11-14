//! Responders (what a handler may return) and errors (`ResponseError`).
//! Run: cargo run -p actix_web_basics --example responders_and_errors

use std::fmt;

use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::test::{self, TestRequest};
use actix_web::{
    App, Either, HttpRequest, HttpResponse, Responder, ResponseError, error, get, web,
};
use serde::Serialize;

// --- Responders -----------------------------------------------------------

#[get("/text")]
async fn text() -> &'static str {
    "plain text" // &str, String: 200, text/plain
}

#[derive(Serialize)]
struct Book {
    title: &'static str,
}

#[get("/json")]
async fn json() -> web::Json<Book> {
    web::Json(Book { title: "Emma" }) // 200, application/json
}

#[get("/built")]
async fn built() -> HttpResponse {
    HttpResponse::Accepted() // the builder: status, headers, body
        .insert_header(("x-hint", "built by hand"))
        .body("queued")
}

#[get("/customised")]
async fn customised() -> impl Responder {
    // Any responder, with a different status or extra headers.
    "teapot".customize().with_status(StatusCode::IM_A_TEAPOT)
}

#[get("/either/{n}")]
async fn either(n: web::Path<u32>) -> Either<web::Json<Book>, HttpResponse> {
    // Two different responder types from one handler.
    if *n == 1 {
        Either::Left(web::Json(Book { title: "Emma" }))
    } else {
        Either::Right(HttpResponse::NotFound().body("no such book"))
    }
}

/// Your own type can be a responder.
struct Greeting(String);

impl Responder for Greeting {
    type Body = BoxBody;
    fn respond_to(self, _req: &HttpRequest) -> HttpResponse {
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(format!("<h1>{}</h1>", self.0))
    }
}

#[get("/greeting")]
async fn greeting() -> Greeting {
    Greeting("Hello".into())
}

// --- Errors ---------------------------------------------------------------

/// A custom error. `ResponseError` needs `Debug + Display`, then says how the
/// error becomes a response. Both methods have defaults (500, and the
/// `Display` text as the body), so `impl ResponseError for ShopError {}`
/// alone would compile.
#[derive(Debug)]
enum ShopError {
    SoldOut(&'static str),
    Closed,
}

impl fmt::Display for ShopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShopError::SoldOut(item) => write!(f, "{item} is sold out"),
            ShopError::Closed => write!(f, "the shop is closed"),
        }
    }
}

impl ResponseError for ShopError {
    fn status_code(&self) -> StatusCode {
        match self {
            ShopError::SoldOut(_) => StatusCode::CONFLICT,
            ShopError::Closed => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .json(serde_json::json!({ "error": self.to_string() }))
    }
}

#[get("/buy/{item}")]
async fn buy(item: web::Path<String>) -> Result<String, ShopError> {
    match item.as_str() {
        "bread" => Ok("one bread".into()),
        "cake" => Err(ShopError::SoldOut("cake")), // `?` works the same way
        _ => Err(ShopError::Closed),
    }
}

#[get("/quick/{n}")]
async fn quick(n: web::Path<String>) -> actix_web::Result<String> {
    // `actix_web::Result<T>` = `Result<T, actix_web::Error>`; the `error::Error*`
    // helpers wrap any error with a status, for quick one-offs.
    let n: u32 = n.parse().map_err(error::ErrorBadRequest)?;
    Ok(format!("{n} doubled is {}", n * 2))
}

#[actix_web::main]
async fn main() {
    let app = test::init_service(
        App::new()
            .service(text)
            .service(json)
            .service(built)
            .service(customised)
            .service(either)
            .service(greeting)
            .service(buy)
            .service(quick),
    )
    .await;

    for path in [
        "/text",
        "/json",
        "/built",
        "/customised",
        "/either/1",
        "/either/2",
        "/greeting",
        "/buy/bread",
        "/buy/cake",
        "/buy/milk",
        "/quick/21",
        "/quick/lots",
    ] {
        let response = test::call_service(&app, TestRequest::get().uri(path).to_request()).await;
        let status = response.status().as_u16();
        let kind = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("-")
            .to_string();
        let body = test::read_body(response).await;
        println!(
            "{path:<12} -> {status} {kind:<26} {}",
            String::from_utf8_lossy(&body)
        );
    }
}
