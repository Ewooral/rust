//! Book 21.2, "Improving Throughput with a Thread Pool": the ThreadPool on
//! its own, no network.
//! Run: cargo run -p web_server --example step5_thread_pool
//!
//! 3 workers, 7 jobs of 200 ms each. The workers take jobs from one shared
//! queue, so the jobs finish in three waves (3 + 3 + 1), about 600 ms in
//! total instead of 1.4 s on a single thread.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use web_server::ThreadPool;

fn main() {
    let start = Instant::now();
    let finished = Arc::new(Mutex::new(Vec::new()));

    {
        let pool = ThreadPool::new(3);
        println!("pool started with {} workers", pool.size());

        for job in 1..=7 {
            let finished = Arc::clone(&finished);
            // execute takes a closure, just like thread::spawn, but no new
            // thread is created: an existing worker runs it.
            pool.execute(move || {
                thread::sleep(Duration::from_millis(200));
                finished.lock().unwrap().push((job, start.elapsed()));
            });
        }
        println!(
            "all 7 jobs queued after {:.0?}; execute never blocks",
            start.elapsed()
        );
        // `pool` is dropped at the end of this block. Drop closes the queue
        // and waits for every worker: watch the "Shutting down" lines.
    }

    println!("\njob  finished after");
    for (job, at) in finished.lock().unwrap().iter() {
        println!("{job:>3}  {:.1?}", at);
    }
    println!("total {:.1?}", start.elapsed());
}
