//! Shared state with `web::Data`, and the per-worker factory trap.
//! Run: cargo run -p actix_web_basics --example shared_state
//!
//! `HttpServer::new(factory)` calls `factory` once PER WORKER THREAD. Data
//! created inside the factory is therefore per worker; data created outside
//! and cloned in (an `Arc` under the hood) is shared by all of them.

use std::net::TcpListener;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use actix_web::{App, HttpServer, Responder, get, web};
use actix_web_basics::client;

/// Shared by everyone: atomics and a Mutex give safe shared mutation.
struct Shared {
    hits: AtomicUsize,
    visitors: Mutex<Vec<String>>,
}

/// One per worker thread: created inside the factory.
struct PerWorker {
    hits: AtomicUsize,
}

#[get("/visit/{name}")]
async fn visit(
    shared: web::Data<Shared>,
    local: web::Data<PerWorker>,
    name: web::Path<String>,
) -> impl Responder {
    let total = shared.hits.fetch_add(1, Ordering::SeqCst) + 1;
    let mine = local.hits.fetch_add(1, Ordering::SeqCst) + 1;
    shared.visitors.lock().unwrap().push(name.into_inner()); // lock, push, unlock
    format!("{total} {mine}")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let factory_calls = web::Data::new(AtomicUsize::new(0));
    let shared = web::Data::new(Shared {
        hits: AtomicUsize::new(0),
        visitors: Mutex::new(Vec::new()),
    });

    let listener = TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    let (shared_in, calls_in) = (shared.clone(), factory_calls.clone());
    let server = HttpServer::new(move || {
        calls_in.fetch_add(1, Ordering::SeqCst);
        App::new()
            .app_data(shared_in.clone()) // the SAME Arc for every worker
            .app_data(web::Data::new(PerWorker {
                hits: AtomicUsize::new(0), // a NEW one for each worker
            }))
            .service(visit)
    })
    .workers(3)
    .listen(listener)?
    .run();
    let handle = server.handle();
    let running = actix_web::rt::spawn(server);

    for name in ["ama", "kofi", "esi", "yaw", "abena", "kwame"] {
        let response = client::get(addr, &format!("/visit/{name}")).await?;
        let (total, mine) = response.body.split_once(' ').unwrap_or_default();
        println!("visit {name:<6} -> shared count {total}, this worker's count {mine}");
    }
    handle.stop(true).await;
    running.await??;

    println!(
        "the factory ran {} times (once per worker)",
        factory_calls.load(Ordering::SeqCst)
    );
    println!("shared hits: {}", shared.hits.load(Ordering::SeqCst));
    println!("visitors: {:?}", shared.visitors.lock().unwrap());
    println!("(the per-worker counts depend on which worker took each connection)");
    Ok(())
}
