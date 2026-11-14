//! The thread pool (Book 21.2 and 21.3).
//!
//! `ThreadPool::new(4)` starts four `Worker` threads up front. `execute`
//! sends a job down a channel; whichever worker is free takes it. Dropping
//! the pool closes the channel, lets every worker finish its current job and
//! then joins them all: a graceful shutdown.

use std::fmt;
use std::panic::{self, AssertUnwindSafe};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

/// A job is any closure that can run once, on another thread, and owns
/// everything it uses. A type alias keeps the long type readable.
type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    /// `Option` so `Drop` can `take()` the sender and drop it first.
    sender: Option<mpsc::Sender<Job>>,
}

/// Returned by [`ThreadPool::build`] when asked for zero threads.
#[derive(Debug, PartialEq, Eq)]
pub struct PoolCreationError;

impl fmt::Display for PoolCreationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a thread pool needs at least one thread")
    }
}

impl std::error::Error for PoolCreationError {}

impl ThreadPool {
    /// Creates a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0, "a thread pool needs at least one thread");
        Self::start(size)
    }

    /// Like [`ThreadPool::new`], but returns an error instead of panicking,
    /// for when the size comes from a user or a config file.
    pub fn build(size: usize) -> Result<ThreadPool, PoolCreationError> {
        if size == 0 {
            return Err(PoolCreationError);
        }
        Ok(Self::start(size))
    }

    fn start(size: usize) -> ThreadPool {
        let (sender, receiver) = mpsc::channel();
        // One receiver, shared by every worker: Arc to share ownership,
        // Mutex so only one worker takes a job at a time.
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// The number of worker threads.
    pub fn size(&self) -> usize {
        self.workers.len()
    }

    /// Runs `f` on the next free worker. The bounds are the same as
    /// `thread::spawn`'s: `FnOnce` (runs once), `Send` (may move to another
    /// thread) and `'static` (borrows nothing that could disappear first).
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender
            .as_ref()
            .expect("the sender only goes away in drop")
            .send(job)
            .expect("workers only stop after the sender is dropped");
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // 1. Close the channel. Workers finish their current job, then
        //    `recv` returns Err and their loops end.
        drop(self.sender.take());

        // 2. Wait for each worker. `drain(..)` MOVES each Worker out of the
        //    Vec, because `join` needs to own its JoinHandle.
        for worker in self.workers.drain(..) {
            println!("Shutting down worker {}", worker.id);
            worker
                .thread
                .join()
                .expect("a worker thread never panics: jobs are caught");
        }
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                // The lock guard is a temporary, dropped at the end of THIS
                // statement, so other workers can take jobs while ours runs.
                // (`while let Ok(job) = receiver.lock().unwrap().recv()`
                // would hold the lock for the whole loop body.)
                let message = receiver
                    .lock()
                    .expect("no thread panics while holding the lock")
                    .recv();

                match message {
                    Ok(job) => {
                        println!("Worker {id} got a job; executing.");
                        // A panicking job must not kill the worker, or the
                        // pool would shrink with every bug.
                        if panic::catch_unwind(AssertUnwindSafe(job)).is_err() {
                            eprintln!("Worker {id}: the job panicked; carrying on.");
                        }
                    }
                    Err(_) => {
                        println!("Worker {id} disconnected; shutting down.");
                        break;
                    }
                }
            }
        });

        Worker { id, thread }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    const PATIENCE: Duration = Duration::from_secs(5);

    #[test]
    fn runs_every_job() {
        let pool = ThreadPool::new(3);
        let (tx, rx) = mpsc::channel();
        for n in 0..10 {
            let tx = tx.clone();
            pool.execute(move || tx.send(n * n).unwrap());
        }
        let mut squares: Vec<i32> = (0..10)
            .map(|_| rx.recv_timeout(PATIENCE).unwrap())
            .collect();
        squares.sort();
        assert_eq!(squares, [0, 1, 4, 9, 16, 25, 36, 49, 64, 81]);
    }

    #[test]
    fn drop_waits_for_queued_jobs() {
        let done = Arc::new(AtomicUsize::new(0));
        {
            let pool = ThreadPool::new(2);
            for _ in 0..6 {
                let done = Arc::clone(&done);
                pool.execute(move || {
                    thread::sleep(Duration::from_millis(20));
                    done.fetch_add(1, Ordering::SeqCst);
                });
            }
        } // pool dropped here: blocks until all six jobs are done
        assert_eq!(done.load(Ordering::SeqCst), 6);
    }

    #[test]
    fn jobs_run_in_parallel() {
        // Two jobs that each wait for the other: possible only if two
        // workers run at the same time. Timeouts stop a broken pool hanging.
        let pool = ThreadPool::new(2);
        let (a_tx, a_rx) = mpsc::channel();
        let (b_tx, b_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();
        let done_a = done_tx.clone();
        pool.execute(move || {
            a_tx.send(()).unwrap();
            done_a.send(b_rx.recv_timeout(PATIENCE).is_ok()).unwrap();
        });
        pool.execute(move || {
            b_tx.send(()).unwrap();
            done_tx.send(a_rx.recv_timeout(PATIENCE).is_ok()).unwrap();
        });
        assert!(done_rx.recv_timeout(PATIENCE * 2).unwrap());
        assert!(done_rx.recv_timeout(PATIENCE * 2).unwrap());
    }

    #[test]
    fn a_panicking_job_does_not_kill_its_worker() {
        let pool = ThreadPool::new(1);
        pool.execute(|| panic!("a buggy request handler"));
        let (tx, rx) = mpsc::channel();
        pool.execute(move || tx.send("still alive").unwrap());
        assert_eq!(rx.recv_timeout(PATIENCE), Ok("still alive"));
    }

    #[test]
    fn build_rejects_zero_threads() {
        assert_eq!(ThreadPool::build(0).err(), Some(PoolCreationError));
        assert_eq!(ThreadPool::build(2).map(|p| p.size()), Ok(2));
    }

    #[test]
    #[should_panic(expected = "at least one thread")]
    fn new_panics_on_zero_threads() {
        ThreadPool::new(0);
    }
}
