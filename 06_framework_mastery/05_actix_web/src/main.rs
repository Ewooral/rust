//! The notes API on Actix-web.
//!
//! Run: cargo run -p actix_web_basics            (http://127.0.0.1:8080)
//!      cargo run -p actix_web_basics -- 8081    (another port)
//! Stop it with Ctrl+C: Actix catches the signal and shuts down gracefully.

use std::net::TcpListener;

use actix_web_basics::{AppState, init_logging, serve};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logging();
    let port: u16 = std::env::args()
        .nth(1)
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    println!("notes API on http://{}", listener.local_addr()?);
    println!("try: curl -i http://127.0.0.1:{port}/api/notes");
    serve(listener, AppState::new(), 2)?.await
}
