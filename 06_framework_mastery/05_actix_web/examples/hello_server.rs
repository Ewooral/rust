//! The smallest Actix-web server: `HttpServer` + `App` + one route, on a free port.
//! Run: cargo run -p actix_web_basics --example hello_server
//!
//! It starts, answers two requests from its own client, and stops itself.

use std::net::TcpListener;

use actix_web::{App, HttpResponse, HttpServer, Responder, get, web};
use actix_web_basics::client;

#[get("/")]
async fn hello() -> impl Responder {
    "Hello, world!"
}

async fn greet(name: web::Path<String>) -> impl Responder {
    HttpResponse::Ok().body(format!("Hello, {name}!"))
}

#[actix_web::main] // starts the Actix runtime (a Tokio runtime) for `main`
async fn main() -> std::io::Result<()> {
    // Bind first, on port 0: the OS picks a free port, so this never clashes.
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;

    // `HttpServer::new` takes a FACTORY closure, and calls it once per worker
    // thread: each worker builds its own `App`.
    let server = HttpServer::new(|| {
        App::new()
            .service(hello) // a handler with a route macro
            .route("/greet/{name}", web::get().to(greet)) // or a plain async fn
    })
    .workers(1)
    .listen(listener)?
    .run(); // a `Server`: a future that runs until the server stops
    let handle = server.handle();
    let running = actix_web::rt::spawn(server);
    println!("listening on http://{addr}");

    for path in ["/", "/greet/Ama"] {
        let response = client::get(addr, path).await?;
        println!("GET {path:<11} -> {} {:?}", response.status, response.body);
    }

    handle.stop(true).await; // graceful: finish what's in flight, then stop
    running.await??;
    println!("server stopped");
    Ok(())
}
