//! Book 21.2, "Spawning a Thread for Each Request": the quick fix the Book
//! shows first, and why it isn't the final answer.
//! Run: cargo run -p web_server --example step4_thread_per_request
//!
//! Same two browsers as step 3. Now each connection gets its own new
//! thread, so B no longer waits for A.

use std::net::TcpListener;
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use web_server::{Router, client, handle_connection};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let router = Arc::new(Router::new(
        Path::new(env!("CARGO_MANIFEST_DIR")),
        Duration::from_secs(1),
    ));

    let start = Instant::now();
    let a = thread::spawn(move || {
        client::get(addr, "/sleep").unwrap();
        println!("A (/sleep) done after {:.1?}", start.elapsed());
    });
    thread::sleep(Duration::from_millis(100));
    let b = thread::spawn(move || {
        client::get(addr, "/").unwrap();
        println!(
            "B (/)      done after {:.1?}  <- no waiting",
            start.elapsed()
        );
    });

    let mut handlers = Vec::new();
    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();
        let router = Arc::clone(&router);
        // A brand-new thread per connection.
        handlers.push(thread::spawn(move || {
            handle_connection(stream, &router).unwrap();
        }));
    }
    for handler in handlers {
        handler.join().unwrap();
    }
    a.join().unwrap();
    b.join().unwrap();

    // The catch: there is NO LIMIT. 10,000 connections at once means 10,000
    // threads, each with its own stack, and the machine falls over. That is
    // an easy denial-of-service attack. A thread pool caps the number of
    // threads and queues the rest (steps 5 and 6).
    println!("\nWorks, but 10,000 visitors would mean 10,000 threads: use a pool.");
}
