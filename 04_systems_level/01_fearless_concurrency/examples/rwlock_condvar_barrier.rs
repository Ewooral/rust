//! Beyond the Book: three more std synchronisation tools.
//!
//! - `RwLock`: many readers OR one writer.
//! - `Condvar`: sleep until another thread says "something changed".
//! - `Barrier`: wait until N threads have all reached the same point.
//!
//! Run: cargo run -p fearless_concurrency --example rwlock_condvar_barrier

use std::collections::VecDeque;
use std::sync::{Arc, Barrier, Condvar, Mutex, RwLock};
use std::thread;
use std::time::Duration;

fn main() {
    rwlock();
    condvar();
    barrier();
}

fn rwlock() {
    println!("--- RwLock ---");
    let config = RwLock::new(String::from("theme=light"));

    // Two read guards at the SAME time: fine, readers share.
    let r1 = config.read().unwrap();
    let r2 = config.read().unwrap();
    println!("two readers at once: {r1} / {r2}");
    // While anyone reads, a writer has to wait.
    println!("try_write while reading: {}", config.try_write().is_ok());
    drop((r1, r2));

    // Many reader threads and one writer thread.
    let config = Arc::new(config);
    let mut handles = Vec::new();
    for id in 0..3 {
        let config = Arc::clone(&config);
        handles.push(thread::spawn(move || {
            let value = config.read().unwrap();
            println!("  reader {id} sees {value}");
        }));
    }
    {
        let config = Arc::clone(&config);
        handles.push(thread::spawn(move || {
            *config.write().unwrap() = String::from("theme=dark");
            println!("  writer changed the theme");
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    println!("final config: {}", config.read().unwrap());
}

fn condvar() {
    println!("\n--- Condvar: a tiny job queue ---");
    // The queue and the "new job!" bell travel together.
    let shared = Arc::new((Mutex::new(VecDeque::new()), Condvar::new()));

    let worker = {
        let shared = Arc::clone(&shared);
        thread::spawn(move || {
            let (queue, bell) = &*shared;
            let mut done = 0;
            loop {
                let mut jobs = queue.lock().unwrap();
                // Sleep (and release the lock!) until the queue isn't empty.
                // `wait_while` re-checks the condition, so spurious wake-ups
                // are handled for you.
                jobs = bell.wait_while(jobs, |q| q.is_empty()).unwrap();
                let job: Option<u32> = jobs.pop_front().unwrap();
                drop(jobs); // unlock before doing the work
                match job {
                    Some(n) => {
                        println!("  worker: job {n}");
                        done += 1;
                    }
                    None => return done, // `None` means "no more jobs"
                }
            }
        })
    };

    let (queue, bell) = &*shared;
    for n in 1..=3 {
        thread::sleep(Duration::from_millis(5));
        queue.lock().unwrap().push_back(Some(n));
        bell.notify_one(); // ring the bell: wake the sleeping worker
    }
    queue.lock().unwrap().push_back(None);
    bell.notify_one();
    println!("worker finished {} jobs", worker.join().unwrap());
}

fn barrier() {
    println!("\n--- Barrier ---");
    let n = 3;
    let barrier = Arc::new(Barrier::new(n));
    let handles: Vec<_> = (0..n)
        .map(|id| {
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(3 * id as u64));
                println!("  thread {id}: phase 1 done, waiting");
                // Nobody passes until all 3 have arrived. Exactly one thread
                // is told it's the "leader", handy for one-off work.
                let leader = barrier.wait().is_leader();
                println!(
                    "  thread {id}: phase 2{}",
                    if leader { " (leader)" } else { "" }
                );
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    println!("every 'phase 1' line printed before any 'phase 2' line");
}
