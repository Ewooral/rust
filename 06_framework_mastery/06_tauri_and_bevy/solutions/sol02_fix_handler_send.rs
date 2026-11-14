//! Solution to exercise 2: make it compile (E0277), twice.
//!
//! A command table like Tauri's: names mapped to boxed closures. The
//! "backend" runs on its own thread and receives (command, argument) pairs
//! over a channel. Tauri runs commands on other threads too, which is why
//! a `#[tauri::command]` and the state it uses must be thread-safe.
//!
//! 1. `type Handler = Box<dyn Fn(&str) -> String + Send + Sync>;`
//!    Send: the box may move to another thread. Sync: several threads may
//!    use it at once through a shared reference (Tauri shares its handlers).
//! 2. `Rc<Cell<u32>>` is neither Send nor Sync: two threads updating Rc's
//!    reference count (or the Cell) at once could corrupt it. An
//!    `Arc<AtomicU32>` does the same job with thread-safe operations.
//!
//! The same rules are why Tauri state is usually wrapped in a `Mutex`
//! (`app.manage(Mutex::new(data))`), and why our `States` only accepts
//! `Send + Sync` values.
//!
//! Run: cargo run -p tauri_and_bevy --features solutions --bin sol02_fix_handler_send

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;

type Handler = Box<dyn Fn(&str) -> String + Send + Sync>;

fn main() {
    let calls = Arc::new(AtomicU32::new(0));

    let mut handlers: HashMap<&'static str, Handler> = HashMap::new();
    handlers.insert("greet", Box::new(|name| format!("Hello, {name}!")));
    handlers.insert("shout", Box::new(|text| text.to_uppercase()));
    let counter = Arc::clone(&calls);
    handlers.insert(
        "count",
        Box::new(move |_| {
            let now = counter.fetch_add(1, Ordering::SeqCst) + 1;
            format!("count has been called {now} time(s)")
        }),
    );

    let (to_backend, inbox) = mpsc::channel::<(String, String)>();
    let backend = thread::spawn(move || {
        for (cmd, arg) in inbox {
            match handlers.get(cmd.as_str()) {
                Some(handler) => println!("{cmd}({arg:?}) -> {}", handler(&arg)),
                None => println!("{cmd}: command not found"),
            }
        }
    });

    for (cmd, arg) in [
        ("greet", "Ama"),
        ("shout", "hello"),
        ("count", ""),
        ("count", ""),
        ("fly", ""),
    ] {
        to_backend
            .send((cmd.to_string(), arg.to_string()))
            .expect("the backend is running");
    }
    drop(to_backend);
    backend.join().expect("the backend finished");
    println!(
        "main thread sees {} call(s) to count",
        calls.load(Ordering::SeqCst)
    );
}
