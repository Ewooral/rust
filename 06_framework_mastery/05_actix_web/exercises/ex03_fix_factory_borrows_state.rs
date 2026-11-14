//! Exercise 3: make it compile (E0373): give every worker the SAME state.
//!
//! `HttpServer::new` takes a factory closure and calls it once per worker
//! thread, possibly long after `main`'s locals are gone, so the closure must
//! own what it uses. This one only borrows `visits`.
//!
//! Fix it so that all workers share ONE counter: the closure should own a
//! handle to `visits` (`web::Data` is an `Arc` inside, so cloning it is cheap
//! and every clone points at the same counter).
//!
//! The compiler suggests `move`. Try it, and read the next error (E0382):
//! `main` still wants `visits` at the end. So move a CLONE into the closure.
//!
//! Don't move `web::Data::new(...)` INTO the closure: that compiles, but each
//! worker would get its own counter. When it's right, `main` prints
//! `total visits: 6` whichever worker answered each request.
//!
//! Run: cargo run -p actix_web_basics --features exercises --bin ex03_fix_factory_borrows_state

use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};

use actix_web::{App, HttpServer, get, web};
use actix_web_basics::client;

#[get("/")]
async fn visit(visits: web::Data<AtomicUsize>) -> String {
    let n = visits.fetch_add(1, Ordering::SeqCst) + 1;
    format!("visit number {n}")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let visits = web::Data::new(AtomicUsize::new(0));
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;

    let server = HttpServer::new(|| App::new().app_data(visits.clone()).service(visit))
        .workers(3)
        .listen(listener)?
        .run();
    let handle = server.handle();
    let running = actix_web::rt::spawn(server);

    for _ in 0..6 {
        println!("{}", client::get(addr, "/").await?.body);
    }
    handle.stop(true).await;
    running.await??;
    println!("total visits: {}", visits.load(Ordering::SeqCst));
    Ok(())
}
