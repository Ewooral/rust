//! Book 16.1: using `move` closures with threads, and why `thread::spawn`
//! needs `F: Send + 'static`.
//!
//! Run: cargo run -p fearless_concurrency --example move_closures

use std::thread;

fn main() {
    // 1. The Book's example. Without `move`, the closure would only BORROW
    // `v`, and the thread might outlive `main`'s local variable:
    //     let handle = thread::spawn(|| println!("{v:?}"));
    //     // E0373: closure may outlive the current function, but it borrows `v`
    let v = vec![1, 2, 3];
    let handle = thread::spawn(move || {
        println!("1. here's a vector: {v:?}");
    });
    // `v` now belongs to the thread, so main can't touch it:
    // drop(v); // E0382: use of moved value: `v`
    handle.join().unwrap();

    // 2. Even a plain number needs `move`: without it the closure borrows
    // `n` (E0373 again). With `move`, the closure gets its own copy.
    let n = 21;
    let doubled = thread::spawn(move || n * 2).join().unwrap();
    println!(
        "2. {n} doubled on another thread is {doubled} (main still has n: Copy types are copied)"
    );

    // 3. Want to keep using the data? Clone BEFORE the move, so the thread
    // owns a copy and main keeps the original.
    let names = vec![String::from("Ama"), String::from("Kofi")];
    let names_for_thread = names.clone();
    let handle = thread::spawn(move || names_for_thread.join(" & "));
    println!(
        "3. thread built {:?}; main still has {names:?}",
        handle.join().unwrap()
    );

    // 4. Move ownership in, get ownership back out through the return value.
    let mut list = vec![5, 3, 9, 1];
    list = thread::spawn(move || {
        list.sort();
        list
    })
    .join()
    .unwrap();
    println!("4. sorted on another thread and handed back: {list:?}");

    // 5. Why `'static`? The signature is
    //     pub fn spawn<F, T>(f: F) -> JoinHandle<T>
    //     where F: FnOnce() -> T + Send + 'static, T: Send + 'static
    // `'static` here means "holds no short-lived borrows" (lesson 03/01), not
    // "lives forever". An owned String qualifies, and so does a &'static str.
    let motto: &'static str = "fearless concurrency";
    let shout = thread::spawn(move || motto.to_uppercase()).join().unwrap();
    println!("5. {shout}");
    // To borrow local data instead, use scoped threads: see the
    // `scoped_threads` example.
}
