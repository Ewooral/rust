//! Exercise 1: make it compile (E0277).
//!
//! A tiny thread pool with ONE worker. Jobs are boxed closures sent down a
//! channel to the worker thread. But the `Job` type alias forgot to say that
//! a job must be able to travel to another thread, and must not borrow
//! anything that might disappear first.
//!
//! Fix the `Job` alias only. `thread::spawn` asks its closure for
//! `Send + 'static`; the Book writes both on `Job`. (The error message shows
//! that `'static` is already there: a boxed trait object is `'static` unless
//! you say otherwise.) Then read the error again: which line did the
//! compiler blame, and why that one?
//!
//! Run: cargo run -p web_server --features exercises --bin ex01_fix_job_not_send

use std::sync::mpsc;
use std::thread;

type Job = Box<dyn FnOnce()>;

fn main() {
    let (sender, receiver) = mpsc::channel::<Job>();

    let worker = thread::spawn(move || {
        for job in receiver {
            job();
        }
        println!("channel closed; worker stopping");
    });

    for n in 1..=3 {
        sender
            .send(Box::new(move || println!("job {n} ran on the worker")))
            .unwrap();
    }
    drop(sender); // close the channel so the worker's loop ends
    worker.join().unwrap();
}
