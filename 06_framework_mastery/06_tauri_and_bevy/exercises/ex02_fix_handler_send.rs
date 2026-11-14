//! Exercise 2: make it compile (E0277), twice.
//!
//! A command table like Tauri's: names mapped to boxed closures. The
//! "backend" runs on its own thread and receives (command, argument) pairs
//! over a channel. Tauri runs commands on other threads too, which is why
//! a `#[tauri::command]` and the state it uses must be thread-safe.
//!
//! 1. Moving the table into `thread::spawn` fails: a `Box<dyn Fn(..)>`
//!    promises nothing about threads. Add `+ Send + Sync` to the `Handler`
//!    type alias (Send: may move to another thread; Sync: may be SHARED
//!    between threads, as Tauri does).
//! 2. Now the `count` handler is refused: it captures an `Rc<Cell<u32>>`,
//!    and `Rc`'s counter isn't thread-safe. Replace it with a thread-safe
//!    counter (`Arc<AtomicU32>`, or `Arc<Mutex<u32>>`).
//!
//! Run: cargo run -p tauri_and_bevy --features exercises --bin ex02_fix_handler_send

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;

type Handler = Box<dyn Fn(&str) -> String>;

fn main() {
    let calls = Rc::new(Cell::new(0u32));

    let mut handlers: HashMap<&'static str, Handler> = HashMap::new();
    handlers.insert("greet", Box::new(|name| format!("Hello, {name}!")));
    handlers.insert("shout", Box::new(|text| text.to_uppercase()));
    let counter = Rc::clone(&calls);
    handlers.insert(
        "count",
        Box::new(move |_| {
            counter.set(counter.get() + 1);
            format!("count has been called {} time(s)", counter.get())
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
}
