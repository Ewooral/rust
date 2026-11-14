//! Routing: route macros, `route`/`resource`, scopes, guards and the default service.
//! Run: cargo run -p actix_web_basics --example routing
//!
//! No network: `actix_web::test::init_service` builds the app as a service and
//! `call_service` feeds it requests directly.

use actix_web::http::Method;
use actix_web::test::{self, TestRequest};
use actix_web::{App, HttpResponse, Responder, get, guard, post, web};

// 1. Route macros: method + path on the function itself.
#[get("/books")]
async fn list_books() -> impl Responder {
    "all books"
}

#[post("/books")]
async fn add_book() -> impl Responder {
    HttpResponse::Created().body("added")
}

// 2. Plain async fns, wired up in the App instead.
async fn one_book(id: web::Path<u32>) -> impl Responder {
    format!("book {id}")
}

async fn delete_book(id: web::Path<u32>) -> impl Responder {
    format!("deleted book {id}")
}

async fn admin_stats() -> impl Responder {
    "admin stats"
}

async fn not_found() -> impl Responder {
    HttpResponse::NotFound().body("nothing here")
}

#[actix_web::main]
async fn main() {
    let app = test::init_service(
        App::new()
            .service(list_books)
            .service(add_book)
            // One resource (path), several methods: an unmatched method on
            // a `resource` gets 405 Method Not Allowed.
            .service(
                web::resource("/books/{id}")
                    .route(web::get().to(one_book))
                    .route(web::delete().to(delete_book)),
            )
            // A scope: a shared prefix (and its own middleware or data).
            // The guard means the whole scope only matches with this header.
            .service(
                web::scope("/admin")
                    .guard(guard::Header("x-admin", "yes"))
                    .route("/stats", web::get().to(admin_stats)),
            )
            .default_service(web::to(not_found)),
    )
    .await;

    let requests = [
        (Method::GET, "/books", false),
        (Method::POST, "/books", false),
        (Method::GET, "/books/7", false),
        (Method::DELETE, "/books/7", false),
        (Method::PUT, "/books/7", false),
        (Method::GET, "/books/seven", false),
        (Method::GET, "/admin/stats", false),
        (Method::GET, "/admin/stats", true),
        (Method::GET, "/nope", false),
    ];
    for (method, path, admin) in requests {
        let mut request = TestRequest::default().method(method.clone()).uri(path);
        if admin {
            request = request.insert_header(("x-admin", "yes"));
        }
        let response = test::call_service(&app, request.to_request()).await;
        let status = response.status();
        let body = test::read_body(response).await;
        let tag = if admin { " (x-admin)" } else { "" };
        println!(
            "{:<6} {:<12}{tag:<10} -> {} {:?}",
            method.as_str(),
            path,
            status.as_u16(),
            String::from_utf8_lossy(&body)
        );
    }
}
