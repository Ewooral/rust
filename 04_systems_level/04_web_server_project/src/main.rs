//! The Book's final project: a multithreaded web server (chapter 21).
//!
//! Run:  cargo run -p web_server          then open http://127.0.0.1:7878
//!       cargo run -p web_server -- 2     stop gracefully after 2 connections
//!                                         (the Book's `.take(2)`)
//! Test: cargo test -p web_server
//!
//! Try http://127.0.0.1:7878/sleep in one tab and http://127.0.0.1:7878 in
//! another: the second page loads at once, because another worker serves it.
//! Stop the server with Ctrl+C. (Ctrl+C ends the process immediately, so
//! `Drop` never runs; the `-- 2` form shows the graceful shutdown.)

use std::env;
use std::path::Path;
use std::process;
use std::time::Duration;

use web_server::{Router, Server};

const ADDRESS: &str = "127.0.0.1:7878";
const THREADS: usize = 4;

fn main() {
    // Optional argument: how many connections to serve before stopping.
    let limit = match env::args().nth(1).map(|arg| arg.parse::<usize>()) {
        None => None,
        Some(Ok(n)) => Some(n),
        Some(Err(_)) => {
            eprintln!("Usage: web_server [number_of_connections]");
            process::exit(1);
        }
    };

    // The pages live next to Cargo.toml, so this works from any directory.
    let pages = Path::new(env!("CARGO_MANIFEST_DIR"));
    let router = Router::new(pages, Duration::from_secs(5));

    let server = Server::bind(ADDRESS, THREADS, router).unwrap_or_else(|e| {
        eprintln!("Could not listen on {ADDRESS}: {e}");
        process::exit(1);
    });

    match limit {
        Some(n) => println!("Serving {n} connection(s) on http://{ADDRESS}, then stopping."),
        None => println!("Listening on http://{ADDRESS} (Ctrl+C to stop)"),
    }
    server.run(limit);
}
