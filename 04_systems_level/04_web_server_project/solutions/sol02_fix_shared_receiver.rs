//! Solution to exercise 2: make it compile (E0382).
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
//! Answer: while one worker holds the lock, no other worker can take a job.
//! Holding it during a slow job would make the pool single-threaded again.
//!
//! Run: cargo run -p web_server --features solutions --bin sol02_fix_shared_receiver

use std::sync::{Arc, Mutex, mpsc};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

fn main() {
    let (sender, receiver) = mpsc::channel::<Job>();
    let receiver = Arc::new(Mutex::new(receiver));

    let mut workers = Vec::new();
    for id in 0..4 {
        let receiver = Arc::clone(&receiver); // each worker gets its own Arc
        workers.push(thread::spawn(move || {
            loop {
                // The MutexGuard is a temporary: it is dropped at the end of
                // this statement, so the lock is free while the job runs.
                // (`while let Ok(job) = receiver.lock().unwrap().recv()`
                // would keep it locked for the whole loop body.)
                let message = receiver.lock().unwrap().recv();
                match message {
                    Ok(job) => {
                        println!("worker {id} got a job");
                        job();
                    }
                    Err(_) => break, // the sender was dropped: no more jobs
                }
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
