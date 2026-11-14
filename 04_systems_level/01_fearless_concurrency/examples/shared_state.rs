//! Book 16.3: shared-state concurrency with `Mutex<T>` and `Arc<T>`, plus
//! the two classic traps: deadlock and poisoning.
//!
//! Run: cargo run -p fearless_concurrency --example shared_state

use std::sync::{Arc, Mutex, TryLockError};
use std::thread;

fn main() {
    // 1. The Mutex API on one thread. `lock()` returns a MutexGuard: a smart
    // pointer that derefs to the data and UNLOCKS when it is dropped.
    let m = Mutex::new(5);
    {
        let mut num = m.lock().unwrap();
        *num = 6;
    } // guard dropped here: the lock is released
    println!("1. m = {m:?}");

    // 2. Sharing a counter between 10 threads: Arc gives shared OWNERSHIP
    // (atomic reference counting), Mutex gives safe MUTATION.
    // Rc instead of Arc fails to compile:
    //     let counter = Rc::new(Mutex::new(0));
    //     thread::spawn(move || ...counter...)
    //     // E0277: `Rc<std::sync::Mutex<i32>>` cannot be sent between threads safely
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }
    println!(
        "2. result: {} ({} Arc owner left)",
        *counter.lock().unwrap(),
        Arc::strong_count(&counter)
    );

    // 3. Keep the lock for as SHORT a time as possible: copy the value out,
    // then do slow work without holding the lock.
    let scores = Arc::new(Mutex::new(vec![3, 1, 2]));
    let snapshot = scores.lock().unwrap().clone(); // guard dropped at the `;`
    let total: i32 = snapshot.iter().sum(); // slow work, lock is free
    println!("3. total of the snapshot: {total}");

    // 4. Deadlock. A Mutex is NOT re-entrant: calling lock() twice on the
    // same thread would wait forever for itself. `try_lock` shows the lock
    // is taken instead of hanging:
    let guard = scores.lock().unwrap();
    match scores.try_lock() {
        Err(TryLockError::WouldBlock) => {
            println!("4. try_lock: WouldBlock (a second lock() here would deadlock)")
        }
        other => println!("4. unexpected: {other:?}"),
    }
    drop(guard);
    // Two threads deadlock when thread 1 holds A and waits for B while
    // thread 2 holds B and waits for A. The cure: always lock in ONE order.
    let a = Arc::new(Mutex::new(1));
    let b = Arc::new(Mutex::new(2));
    let handles: Vec<_> = (0..2)
        .map(|_| {
            let (a, b) = (Arc::clone(&a), Arc::clone(&b));
            thread::spawn(move || {
                let mut first = a.lock().unwrap(); // every thread: A first...
                let mut second = b.lock().unwrap(); // ...then B
                *first += 1;
                *second += 1;
            })
        })
        .collect();
    for handle in handles {
        handle.join().unwrap();
    }
    println!("4. same lock order in every thread: a = {a:?}, b = {b:?}");

    // 5. Poisoning: if a thread panics while holding the lock, the Mutex is
    // marked "poisoned" and later lock() calls return Err.
    println!("5. (the panic message below is expected)");
    let data = Arc::new(Mutex::new(vec![1, 2, 3]));
    let data_for_thread = Arc::clone(&data);
    let _ = thread::spawn(move || {
        let mut guard = data_for_thread.lock().unwrap();
        guard.push(4);
        panic!("oops, panicked while holding the lock");
    })
    .join();
    match data.lock() {
        Ok(guard) => println!("5. not poisoned: {guard:?}"),
        Err(poisoned) => {
            // The data is still there; you decide whether it's trustworthy.
            let guard = poisoned.into_inner();
            println!("5. poisoned, but recovered the data: {guard:?}");
        }
    }
}
