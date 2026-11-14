//! Book 16.1: creating threads with `thread::spawn` and waiting for them with
//! `join`. Run it a few times: the interleaving changes from run to run,
//! because the operating system decides which thread runs when.
//!
//! Run: cargo run -p fearless_concurrency --example spawn_and_join

use std::thread;
use std::time::Duration;

fn main() {
    println!("this machine can run {} threads at once", cores());

    // 1. spawn + join at the END: both threads take turns, then main waits.
    println!("\n--- 1. join at the end: the two threads interleave ---");
    let handle = thread::spawn(|| {
        for i in 1..5 {
            println!("  spawned: hi number {i}");
            thread::sleep(Duration::from_millis(1));
        }
    });
    for i in 1..3 {
        println!("  main:    hi number {i}");
        thread::sleep(Duration::from_millis(1));
    }
    // `join` blocks main until the spawned thread has finished.
    handle.join().unwrap();

    // 2. Where you call join matters: joining FIRST means no interleaving.
    println!("\n--- 2. join before main's loop: no interleaving ---");
    let handle = thread::spawn(|| {
        for i in 1..4 {
            println!("  spawned: hi number {i}");
            thread::sleep(Duration::from_millis(1));
        }
    });
    handle.join().unwrap();
    for i in 1..3 {
        println!("  main:    hi number {i}");
    }

    // 3. A thread can RETURN a value: JoinHandle<T> hands it back on join.
    println!("\n--- 3. a thread that returns a value ---");
    let handle = thread::Builder::new()
        .name("adder".to_string())
        .spawn(|| {
            let me = thread::current();
            println!("  thread {:?} is adding 1..=100", me.name().unwrap());
            (1..=100).sum::<u32>()
        })
        .expect("the OS refused to create a thread");
    let total: u32 = handle.join().unwrap();
    println!("  main got {total} back from the thread");

    // 4. A panic inside a thread does NOT crash main: join returns Err.
    println!("\n--- 4. a thread that panics (the panic message below is expected) ---");
    let handle = thread::spawn(|| {
        let empty: Vec<i32> = Vec::new();
        empty[0] // index out of bounds: this thread panics
    });
    match handle.join() {
        Ok(value) => println!("  got {value}"),
        Err(_) => println!("  main: the thread panicked, but main carries on"),
    }

    // 5. No join at all: when main returns, the whole process ends and
    // any thread still running is stopped mid-count.
    println!("\n--- 5. no join: main finishes first and the spawned thread is cut off ---");
    thread::spawn(|| {
        for i in 1..100 {
            println!("  spawned: counting {i}");
            thread::sleep(Duration::from_millis(2));
        }
    });
    thread::sleep(Duration::from_millis(5));
    println!("  main: done, so the program ends now (the spawned thread never reaches 99)");
}

fn cores() -> usize {
    thread::available_parallelism().map_or(1, |n| n.get())
}
