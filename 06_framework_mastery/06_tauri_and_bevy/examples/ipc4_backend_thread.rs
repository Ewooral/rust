//! Tauri part 4: the page and the Rust core don't share a thread, they
//! exchange messages. Here the "backend" runs on its own thread and the
//! "page" talks to it through channels, which is why every command, and
//! everything they share, must be `Send + Sync` (exercise 2).
//! Run: cargo run -p tauri_and_bevy --example ipc4_backend_thread

use std::sync::mpsc;
use std::thread;

use serde_json::json;
use tauri_and_bevy::ipc::Frontend;
use tauri_and_bevy::notes_app;

fn main() {
    let (to_backend, backend_inbox) = mpsc::channel::<String>();
    let (to_page, page_inbox) = mpsc::channel::<String>();

    // The Router moves into another thread: this only compiles because its
    // handlers are `Send + Sync` and its state is in a Mutex.
    let backend = thread::spawn(move || {
        let router = notes_app::build_router();
        for message in backend_inbox {
            let reply = router.handle_json(&message);
            to_page.send(reply).expect("the page is listening");
            for event in router.take_events() {
                let json = serde_json::to_string(&event).expect("events serialize");
                to_page.send(json).expect("the page is listening");
            }
        }
        println!("[backend] the page closed: shutting down");
    });

    let mut page = Frontend::new("main");
    for (cmd, args) in [
        ("add_note", json!({ "title": "Sent across threads" })),
        ("list_notes", json!({})),
    ] {
        let message = page.invoke_message(cmd, args);
        println!("[page]    sends    {message}");
        to_backend.send(message).expect("the backend is running");
    }
    drop(to_backend); // closing the channel ends the backend's loop

    for reply in page_inbox {
        println!("[page]    receives {reply}");
    }
    backend.join().expect("the backend thread finished cleanly");
}
