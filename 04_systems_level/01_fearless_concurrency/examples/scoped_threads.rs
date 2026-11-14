//! Beyond the Book: scoped threads with `std::thread::scope` (Rust 1.63+).
//! They may BORROW local data, because the scope guarantees every thread
//! has finished before the borrowed data can go away.
//!
//! Run: cargo run -p fearless_concurrency --example scoped_threads

use std::thread;

fn main() {
    let numbers: Vec<u64> = (1..=1_000).collect();

    // 1. With plain `thread::spawn`, borrowing `numbers` fails (E0373: closure
    // may outlive the current function, but it borrows `numbers`), so you'd
    // reach for `move`, clones or Arc. A scope lets threads just borrow:
    let (evens, odds) = thread::scope(|s| {
        let evens = s.spawn(|| numbers.iter().filter(|n| *n % 2 == 0).sum::<u64>());
        let odds = s.spawn(|| numbers.iter().filter(|n| *n % 2 == 1).sum::<u64>());
        (evens.join().unwrap(), odds.join().unwrap())
    });
    println!(
        "1. evens = {evens}, odds = {odds}, and main still owns all {} numbers",
        numbers.len()
    );

    // 2. Split the work into chunks: one thread per chunk, no Arc, no clone.
    let chunk_sums: Vec<u64> = thread::scope(|s| {
        let handles: Vec<_> = numbers
            .chunks(250)
            .map(|chunk| s.spawn(move || chunk.iter().sum::<u64>()))
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    println!(
        "2. chunk sums {chunk_sums:?} add up to {}",
        chunk_sums.iter().sum::<u64>()
    );

    // 3. MUTABLE borrows work too, as long as no two threads touch the same
    // data. `chunks_mut` hands each thread its own separate slice.
    let mut pixels = vec![10u8; 12];
    thread::scope(|s| {
        for (i, row) in pixels.chunks_mut(4).enumerate() {
            s.spawn(move || {
                for p in row.iter_mut() {
                    *p += i as u8; // row 0 +0, row 1 +1, row 2 +2
                }
            });
        }
    }); // every thread is joined automatically here
    println!("3. brightened rows: {pixels:?}");

    // 4. The borrow checker still prevents data races. Two threads mutating
    // the same variable is rejected:
    //     let mut count = 0;
    //     thread::scope(|s| {
    //         for _ in 0..4 {
    //             s.spawn(|| count += 1);
    //         }
    //     });
    //     // E0499: cannot borrow `count` as mutable more than once at a time
    // Fix: a Mutex or an atomic (see the `atomics` example).
    println!("4. see the commented-out data race: it doesn't compile");

    // 5. Threads you don't join yourself are joined when the scope ends,
    // and a panic in one of them is re-raised there.
    let result = thread::scope(|s| {
        s.spawn(|| println!("5. a thread nobody joins by hand"));
        "the scope waited for it"
    });
    println!("5. {result}");
}
