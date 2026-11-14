//! Exercise 2: make it compile (E0382).
//!
//! `send` MOVES the value into the channel: once it's sent, the thread no
//! longer owns it. Fix the thread WITHOUT `.clone()`, by changing the order
//! of two lines, so it still prints "sending: ..." for every message.
//!
//! Run: cargo run -p fearless_concurrency --features exercises --bin ex02_fix_use_after_send

use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        for n in 1..=3 {
            let message = format!("parcel number {n}");
            tx.send(message).unwrap();
            println!("sending: {message}");
        }
    });

    for received in rx {
        println!("main received: {received}");
    }
}
