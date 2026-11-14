//! Exercise 6: build the ThreadPool yourself.
//!
//! `Worker` is done for you. Implement the three `todo!()`s:
//!   - `ThreadPool::build(size)`: `Err(PoolCreationError)` for size 0
//!     (the Book's suggested alternative to panicking in `new`); otherwise
//!     make a channel, share its receiver with `Arc<Mutex<…>>`, and start
//!     `size` workers.
//!   - `execute(f)`: box the closure and send it down the channel.
//!   - `Drop`: graceful shutdown. Close the channel FIRST, then join every
//!     worker (exercise 3 showed why the order matters).
//!
//! The tests are written; make them pass. If a test hangs, your `drop`
//! joins before closing the channel.
//!
//! Test: cargo test -p web_server --features exercises --bin ex06_thread_pool_build

#![allow(unused_variables, dead_code)]

use std::sync::{Arc, Mutex, mpsc};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

#[derive(Debug, PartialEq, Eq)]
struct PoolCreationError;

struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    fn build(size: usize) -> Result<ThreadPool, PoolCreationError> {
        todo!()
    }

    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        todo!()
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        todo!()
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    /// Takes jobs until the channel is closed, then returns.
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
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
    }
}

fn main() {
    match ThreadPool::build(0) {
        Ok(_) => println!("a pool with no threads?!"),
        Err(e) => println!("build(0) -> {e:?}"),
    }
    let pool = ThreadPool::build(2).expect("two threads is fine");
    for n in 1..=4 {
        pool.execute(move || println!("job {n} on {:?}", thread::current().id()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[test]
    fn build_checks_the_size() {
        assert_eq!(ThreadPool::build(0).err(), Some(PoolCreationError));
        assert_eq!(ThreadPool::build(3).map(|pool| pool.workers.len()), Ok(3));
    }

    #[test]
    fn execute_runs_every_job() {
        let pool = ThreadPool::build(3).unwrap();
        let (tx, rx) = mpsc::channel();
        for n in 0..6 {
            let tx = tx.clone();
            pool.execute(move || tx.send(n).unwrap());
        }
        let mut got: Vec<i32> = (0..6)
            .map(|_| rx.recv_timeout(Duration::from_secs(5)).unwrap())
            .collect();
        got.sort();
        assert_eq!(got, [0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn drop_finishes_every_job_first() {
        let finished = Arc::new(AtomicUsize::new(0));
        {
            let pool = ThreadPool::build(2).unwrap();
            for _ in 0..5 {
                let finished = Arc::clone(&finished);
                pool.execute(move || {
                    thread::sleep(Duration::from_millis(20));
                    finished.fetch_add(1, Ordering::SeqCst);
                });
            }
        } // drop: must wait for all five
        assert_eq!(finished.load(Ordering::SeqCst), 5);
    }
}
