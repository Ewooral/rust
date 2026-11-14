//! The real server, over real TCP, on a free port (`127.0.0.1:0`).

use std::net::TcpListener;

use actix_web_basics::{AppState, client, serve};

#[actix_web::test]
async fn the_real_server_answers_and_stops_gracefully() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let state = AppState::new();
    let server = serve(listener, state.clone(), 2).unwrap();
    let handle = server.handle();
    let running = actix_web::rt::spawn(server);

    let hello = client::get(addr, "/").await.unwrap();
    assert_eq!(
        (hello.status, hello.body.as_str()),
        (200, "Hello from Actix!")
    );

    let created = client::send(addr, "POST", "/api/notes", Some(r#"{"title":"over TCP"}"#))
        .await
        .unwrap();
    assert_eq!(created.status, 201);
    assert_eq!(created.header("location"), Some("/api/notes/1"));
    assert!(created.header("x-request-id").is_some());

    // Two workers, one store: whichever worker answers, the note is there.
    for _ in 0..4 {
        let fetched = client::get(addr, "/api/notes/1").await.unwrap();
        assert_eq!(fetched.json()["title"], "over TCP");
    }
    assert_eq!(state.notes.len(), 1);

    // Graceful stop: stop accepting, finish requests in flight, then return.
    // (We don't then check that a new connection is refused: on Windows a
    // refused connect to localhost takes about 2 s, which would slow the test.)
    handle.stop(true).await;
    running.await.unwrap().unwrap();
}
