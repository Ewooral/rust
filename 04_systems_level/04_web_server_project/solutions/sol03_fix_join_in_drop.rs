//! Solution to exercise 3: make it compile (E0507).
//!
//! Graceful shutdown: when the pool is dropped it should join every worker
//! thread, so no work is cut off. But `JoinHandle::join` takes `self`: it
//! needs to OWN the handle, and `drop` only has `&mut self`.
//!
//! Fix it ONE of these ways:
//!   a) the Book's way: `for worker in self.workers.drain(..)`, which moves
//!      each Worker out of the Vec (leaving it empty);
//!   b) store `thread: Option<JoinHandle<()>>` and `take()` it.
//!
//! Also notice the order inside `drop`: why must the sender be dropped
//! BEFORE joining? (Try swapping the two steps once it compiles. The
//! program will hang: press Ctrl+C.)
//! Answer: a worker only leaves its loop when `recv` returns Err, which
//! happens once the sender is gone. Join first, and you wait forever for a
//! worker that is waiting forever for a job: a deadlock.
//!
//! Run: cargo run -p web_server --features solutions --bin sol03_fix_join_in_drop

use std::sync::{Arc, Mutex, mpsc};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    fn new(size: usize) -> ThreadPool {
        let (sender, receiver) = mpsc::channel::<Job>();
        let receiver = Arc::new(Mutex::new(receiver));
        let workers = (0..size)
            .map(|id| {
                let receiver = Arc::clone(&receiver);
                let thread = thread::spawn(move || {
                    loop {
                        let message = receiver.lock().unwrap().recv();
                        match message {
                            Ok(job) => job(),
                            Err(_) => break,
                        }
                    }
                });
                Worker { id, thread }
            })
            .collect();
        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    fn execute(&self, f: impl FnOnce() + Send + 'static) {
        self.sender.as_ref().unwrap().send(Box::new(f)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take()); // 1. close the channel
        // `drain(..)` hands us each Worker BY VALUE, so we own its JoinHandle
        // and may call `join`, which consumes it.
        for worker in self.workers.drain(..) {
            // 2. wait for each worker
            println!("Shutting down worker {}", worker.id);
            worker.thread.join().unwrap();
        }
    }
}

fn main() {
    let pool = ThreadPool::new(3);
    for n in 1..=5 {
        pool.execute(move || println!("job {n} done"));
    }
    println!("main is done; dropping the pool");
}
