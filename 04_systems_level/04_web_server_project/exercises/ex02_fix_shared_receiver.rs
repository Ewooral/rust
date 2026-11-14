//! Exercise 2: make it compile (E0382).
//!
//! The Book's "compiler-driven development" step: four workers should all
//! take jobs from ONE channel. But a `Receiver` can't be copied, and the
//! first worker's closure takes it, so the second can't.
//!
//! Share it the way the Book does:
//!   - `Arc` so all four workers OWN the receiver together,
//!   - `Mutex` so only ONE worker takes a job at a time.
//!
//! Then each worker does `receiver.lock().unwrap().recv()`.
//! Bonus: why must the lock be released BEFORE the job runs?
//!
//! Run: cargo run -p web_server --features exercises --bin ex02_fix_shared_receiver

use std::sync::mpsc;
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

fn main() {
    let (sender, receiver) = mpsc::channel::<Job>();

    let mut workers = Vec::new();
    for id in 0..4 {
        workers.push(thread::spawn(move || {
            while let Ok(job) = receiver.recv() {
                println!("worker {id} got a job");
                job();
            }
        }));
    }

    for n in 1..=8 {
        sender
            .send(Box::new(move || println!("  job {n} done")))
            .unwrap();
    }
    drop(sender);
    for worker in workers {
        worker.join().unwrap();
    }
}
