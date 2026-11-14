//! Beyond the Book: atomics and memory ordering. An atomic is a number (or
//! bool) that threads can change without a lock, using special CPU
//! instructions. `Ordering` says what OTHER memory the operation keeps in
//! order.
//!
//! Run: cargo run -p fearless_concurrency --example atomics

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

fn main() {
    // 1. A counter shared by 8 threads, no Mutex. `Relaxed` is enough: we
    // only need the count itself to be right, nothing else.
    // (A plain `count += 1` from several threads won't compile: E0499. See
    // the `scoped_threads` example.)
    let hits = AtomicUsize::new(0);
    thread::scope(|s| {
        for _ in 0..8 {
            s.spawn(|| {
                for _ in 0..1_000 {
                    hits.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    });
    println!(
        "1. hits = {} (always exactly 8000)",
        hits.load(Ordering::Relaxed)
    );

    // 2. A stop flag: the worker loops until main says stop.
    let stop = AtomicBool::new(false);
    let rounds = thread::scope(|s| {
        let worker = s.spawn(|| {
            let mut rounds = 0u32;
            while !stop.load(Ordering::Relaxed) {
                rounds += 1;
                thread::sleep(Duration::from_millis(1));
            }
            rounds
        });
        thread::sleep(Duration::from_millis(10));
        stop.store(true, Ordering::Relaxed);
        worker.join().unwrap()
    });
    println!("2. the worker stopped after {rounds} rounds");

    // 3. Release / Acquire: publishing data. The producer writes DATA, then
    // sets READY with Release. A consumer that sees READY == true with
    // Acquire is guaranteed to also see the DATA written before it.
    let data = AtomicU64::new(0);
    let ready = AtomicBool::new(false);
    thread::scope(|s| {
        s.spawn(|| {
            data.store(42, Ordering::Relaxed); // the parcel...
            ready.store(true, Ordering::Release); // ...then the doorbell
        });
        s.spawn(|| {
            while !ready.load(Ordering::Acquire) {
                std::hint::spin_loop(); // hear the doorbell...
            }
            // ...and the parcel is guaranteed to be on the step.
            println!("3. consumer saw data = {}", data.load(Ordering::Relaxed));
        });
    });

    // 4. compare_exchange: "set it to X only if it is still Y". Exactly one
    // thread wins the race to claim the prize.
    let winner = AtomicUsize::new(usize::MAX); // MAX = nobody yet
    thread::scope(|s| {
        for id in 0..4 {
            let winner = &winner;
            s.spawn(move || {
                let claimed = winner
                    .compare_exchange(usize::MAX, id, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok();
                if claimed {
                    println!("4. thread {id} claimed the prize");
                }
            });
        }
    });
    println!("4. winner is thread {}", winner.load(Ordering::SeqCst));

    // 5. fetch_max / fetch_update: lock-free "keep the highest score".
    let high_score = AtomicU64::new(0);
    thread::scope(|s| {
        for score in [30, 75, 12, 60] {
            let high_score = &high_score;
            s.spawn(move || high_score.fetch_max(score, Ordering::Relaxed));
        }
    });
    println!("5. high score = {}", high_score.load(Ordering::Relaxed));

    // SeqCst ("sequentially consistent") is the strongest ordering: every
    // thread agrees on one single order of all SeqCst operations. When in
    // doubt, use SeqCst; when you need speed, measure, and read
    // "Rust Atomics and Locks" (Mara Bos) before relaxing it.
}
