//! Actix's runtime: one single-threaded Tokio runtime per worker, so handlers
//! may hold `!Send` values (an `Rc`) across `.await`. Plus `web::block`.
//! Run: cargo run -p actix_web_basics --example runtime_and_actors

use std::cell::RefCell;
use std::net::TcpListener;
use std::rc::Rc;
use std::time::Duration;

use actix_web::{App, HttpServer, Responder, get, web};
use actix_web_basics::client;

/// Per-worker, NOT thread-safe state: `Rc<RefCell<…>>`. Axum would refuse
/// this (its handlers' futures must be `Send`); Actix allows it, because a
/// request never leaves the worker thread that accepted it.
#[derive(Clone, Default)]
struct WorkerNotes(Rc<RefCell<Vec<String>>>);

#[get("/note/{text}")]
async fn note(notes: web::Data<WorkerNotes>, text: web::Path<String>) -> impl Responder {
    let mine = notes.0.clone(); // an Rc: !Send
    actix_web::rt::time::sleep(Duration::from_millis(5)).await; // ...held across .await
    mine.borrow_mut().push(text.into_inner());
    let thread = std::thread::current().name().unwrap_or("?").to_string();
    format!("{thread} holds {} note(s)", mine.borrow().len())
}

/// Blocking work (a slow hash, a sync library, file I/O) must not run on the
/// worker thread: it would stall every connection on that worker.
/// `web::block` runs it on a separate thread pool and awaits the result.
#[get("/slow-sum/{n}")]
async fn slow_sum(n: web::Path<u64>) -> actix_web::Result<String> {
    let n = n.into_inner();
    let total = web::block(move || {
        std::thread::sleep(Duration::from_millis(20)); // pretend this is heavy
        (1..=n).sum::<u64>()
    })
    .await?;
    Ok(format!("sum 1..={n} = {total}"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("main runs on thread {:?}", std::thread::current().name());

    // `rt::spawn` is Tokio's `spawn_local`: the task stays on this thread, so
    // it may own an Rc too.
    let local = Rc::new(41);
    let answer = actix_web::rt::spawn(async move { *local + 1 })
        .await
        .unwrap_or_default();
    println!("rt::spawn on this thread computed {answer}");

    let listener = TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    let server = HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(WorkerNotes::default())) // one per worker
            .service(note)
            .service(slow_sum)
    })
    .workers(2)
    .listen(listener)?
    .run();
    let handle = server.handle();
    let running = actix_web::rt::spawn(server);

    for text in ["milk", "eggs", "bread", "jam"] {
        let response = client::get(addr, &format!("/note/{text}")).await?;
        println!("GET /note/{text:<5} -> {}", response.body);
    }
    let response = client::get(addr, "/slow-sum/1000").await?;
    println!("GET /slow-sum/1000 -> {}", response.body);

    handle.stop(true).await;
    running.await??;
    Ok(())
}
