//! Middleware: `Logger`, `wrap_fn`, `from_fn`, `DefaultHeaders`, and their order.
//! Run: cargo run -p actix_web_basics --example middleware

use std::time::Instant;

use actix_web::body::MessageBody;
use actix_web::dev::{Service as _, ServiceRequest, ServiceResponse};
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::middleware::{DefaultHeaders, Logger, Next, from_fn};
use actix_web::test::{self, TestRequest};
use actix_web::{App, Error, HttpResponse, web};

/// An async-fn middleware (`from_fn`): it gets the request and `next`, the
/// rest of the chain. It can answer early, or call `next` and change the result.
async fn require_api_key(
    req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let allowed = req
        .headers()
        .get("x-api-key")
        .is_some_and(|key| key == "let-me-in");
    if !allowed {
        println!("  [api key]   refused {}", req.path());
        let refusal = HttpResponse::Unauthorized().body("missing or wrong x-api-key");
        // Answer here: the handler never runs.
        return Ok(req.into_response(refusal).map_into_right_body());
    }
    println!("  [api key]   ok");
    next.call(req)
        .await
        .map(ServiceResponse::map_into_left_body)
}

#[actix_web::main]
async fn main() {
    actix_web_basics::init_logging(); // so the Logger's lines are printed

    let app = test::init_service(
        App::new()
            // `wrap_fn`: a closure middleware. It sees the request first, then
            // (after `srv.call(req)`) the response.
            .wrap_fn(|req, srv| {
                println!("  [timer]     before {}", req.path());
                let started = Instant::now();
                let response = srv.call(req);
                async move {
                    let mut response = response.await?;
                    let took = format!("{}us", started.elapsed().as_micros());
                    println!("  [timer]     after: {}", response.status());
                    if let Ok(value) = HeaderValue::from_str(&took) {
                        response
                            .headers_mut()
                            .insert(HeaderName::from_static("x-took"), value);
                    }
                    Ok(response)
                }
            })
            // Headers added to every response.
            .wrap(DefaultHeaders::new().add(("x-shop", "open")))
            // Registered last, so it runs FIRST (outermost) and logs the
            // finished response.
            .wrap(Logger::new(r#""%r" %s"#))
            .route("/", web::get().to(|| async { "public page" }))
            // Middleware on a scope applies only inside it.
            .service(
                web::scope("/admin")
                    .wrap(from_fn(require_api_key))
                    .route("/stats", web::get().to(|| async { "secret stats" })),
            ),
    )
    .await;

    let requests = [
        ("/", None),
        ("/admin/stats", None),
        ("/admin/stats", Some("let-me-in")),
    ];
    for (path, key) in requests {
        println!("GET {path} (key: {key:?})");
        let mut request = TestRequest::get().uri(path);
        if let Some(key) = key {
            request = request.insert_header(("x-api-key", key));
        }
        let response = test::call_service(&app, request.to_request()).await;
        let status = response.status().as_u16();
        let shop = response.headers().contains_key("x-shop");
        let timed = response.headers().contains_key("x-took");
        let body = test::read_body(response).await;
        println!(
            "  => {status} {:?} (x-shop: {shop}, x-took: {timed})",
            String::from_utf8_lossy(&body)
        );
    }
}
