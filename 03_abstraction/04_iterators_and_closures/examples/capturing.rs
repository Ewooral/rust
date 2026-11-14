//! Book 13.1: how closures capture: borrow, borrow mutably, or take ownership.
//! Run: cargo run -p iterators_and_closures --example capturing
//!
//! A closure captures each variable in the LEAST powerful way its body needs.

use std::thread;

fn main() {
    // 1. Immutable borrow: the body only reads `list`.
    let list = vec![1, 2, 3];
    println!("Before defining closure: {list:?}");
    let only_borrows = || println!("From closure: {list:?}");
    println!("Before calling closure: {list:?}"); // other readers are fine
    only_borrows();
    println!("After calling closure: {list:?}");

    // 2. Mutable borrow: the body changes `list`, so the closure itself must
    //    be `mut`, and nobody else may use `list` until its last call.
    let mut list = vec![1, 2, 3];
    let mut borrows_mutably = || list.push(7);
    // println!("{list:?}"); // E0502: cannot borrow `list` as immutable because it is also borrowed as mutable
    borrows_mutably();
    println!("After calling closure: {list:?}"); // fine: the borrow has ended

    // 3. Move: `move` forces the closure to TAKE OWNERSHIP of what it uses.
    //    Needed when the closure may outlive this function, e.g. a thread:
    //    the main thread might finish and drop `list` first.
    let list = vec![1, 2, 3];
    let handle = thread::spawn(move || println!("From thread: {list:?}"));
    handle.join().expect("thread finished");
    // println!("{list:?}"); // E0382: borrow of moved value: `list`

    // Without `move`, thread::spawn would not compile (E0373: closure may
    // outlive the current function, but it borrows `list`).

    // `move` with Copy values copies them: the original stays usable.
    let count = 5;
    let show = move || println!("captured a copy: {count}");
    show();
    println!("count is still {count}");
}
