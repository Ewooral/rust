//! Extractors: `web::Path`, `web::Query`, `web::Json`, `web::Form`, `HttpRequest`, and their configs.
//! Run: cargo run -p actix_web_basics --example extractors

use actix_web::test::{self, TestRequest};
use actix_web::{App, HttpRequest, HttpResponse, Responder, ResponseError, error, get, post, web};
use serde::Deserialize;
use serde_json::json;

/// Path segments into a tuple…
#[get("/users/{user}/posts/{post}")]
async fn by_tuple(path: web::Path<(String, u32)>) -> impl Responder {
    let (user, post) = path.into_inner();
    format!("tuple: user={user} post={post}")
}

/// …or into a struct, matched by the names in `{…}`.
#[derive(Deserialize)]
struct PostPath {
    user: String,
    post: u32,
}

#[get("/v2/users/{user}/posts/{post}")]
async fn by_struct(path: web::Path<PostPath>) -> impl Responder {
    format!("struct: user={} post={}", path.user, path.post)
}

/// `?q=rust&page=2`: missing optional fields are fine, unknown ones ignored
/// (unless the struct says `deny_unknown_fields`).
#[derive(Deserialize)]
struct Search {
    q: String,
    page: Option<u32>,
}

#[get("/search")]
async fn search(query: web::Query<Search>) -> impl Responder {
    format!("search: q={} page={}", query.q, query.page.unwrap_or(1))
}

/// A JSON body. Needs `content-type: application/json`.
#[derive(Deserialize)]
struct NewUser {
    name: String,
    age: u8,
}

#[post("/users")]
async fn create(user: web::Json<NewUser>) -> impl Responder {
    HttpResponse::Created().body(format!("json: {} is {}", user.name, user.age))
}

/// An HTML form body (`application/x-www-form-urlencoded`).
#[post("/login")]
async fn login(form: web::Form<Search>) -> impl Responder {
    format!("form: q={}", form.q)
}

/// The raw request, for anything else: headers, method, peer address…
/// Several extractors in one handler are fine; at most one may read the body.
#[get("/whoami")]
async fn whoami(req: HttpRequest, query: web::Query<Search>) -> impl Responder {
    let agent = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("?");
    format!(
        "{} {} as {agent:?}, q={}",
        req.method(),
        req.path(),
        query.q
    )
}

#[actix_web::main]
async fn main() {
    let app = test::init_service(
        App::new()
            // Extractor settings live in app data. This one caps JSON bodies
            // at 64 bytes and words its errors our way (keeping the status).
            .app_data(
                web::JsonConfig::default()
                    .limit(64)
                    .error_handler(|err, _req| {
                        let response =
                            HttpResponse::build(err.status_code()).body(format!("bad JSON: {err}"));
                        error::InternalError::from_response(err, response).into()
                    }),
            )
            .service(by_tuple)
            .service(by_struct)
            .service(search)
            .service(create)
            .service(login)
            .service(whoami),
    )
    .await;

    let requests = [
        TestRequest::get().uri("/users/ama/posts/7"),
        TestRequest::get().uri("/v2/users/ama/posts/7"),
        TestRequest::get().uri("/users/ama/posts/seven"),
        TestRequest::get().uri("/search?q=rust&page=2"),
        TestRequest::get().uri("/search?page=2"),
        TestRequest::post()
            .uri("/users")
            .set_json(json!({ "name": "Ama", "age": 9 })),
        TestRequest::post()
            .uri("/users")
            .set_json(json!({ "name": "Ama", "age": 900 })),
        TestRequest::post()
            .uri("/users")
            .set_payload(r#"{"name":"Ama","age":9}"#),
        TestRequest::post()
            .uri("/users")
            .set_json(json!({ "name": "A".repeat(80), "age": 9 })),
        TestRequest::post().uri("/login").set_form([("q", "rust")]),
        TestRequest::get()
            .uri("/whoami?q=hi")
            .insert_header(("user-agent", "curl/8.5.0")),
    ];
    for request in requests {
        let request = request.to_request();
        let line = format!("{} {}", request.method(), request.uri());
        let response = test::call_service(&app, request).await;
        let status = response.status().as_u16();
        let body = test::read_body(response).await;
        println!("{line:<32} -> {status} {}", String::from_utf8_lossy(&body));
    }
}
