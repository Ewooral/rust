//! Book 16.2: transferring data between threads with channels (`mpsc`:
//! multiple producer, single consumer).
//!
//! Run: cargo run -p fearless_concurrency --example message_passing

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    // 1. One value. `channel()` returns (transmitter, receiver).
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let val = String::from("hi");
        tx.send(val).unwrap();
        // `send` MOVED `val` into the channel. Using it now is an error:
        // println!("val is {val}"); // E0382: borrow of moved value: `val`
    });
    // `recv` blocks until a value arrives (or every sender is gone).
    let received = rx.recv().unwrap();
    println!("1. got: {received}");

    // 2. Several values, with pauses. The receiver is an ITERATOR: the loop
    // ends when the sender is dropped (here, when the thread finishes).
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for word in ["hi", "from", "the", "thread"] {
            tx.send(word.to_string()).unwrap();
            thread::sleep(Duration::from_millis(5));
        }
    }); // the thread ends, `tx` is dropped, and the `for` loop below finishes
    for received in rx {
        println!("2. got: {received}");
    }

    // 3. Multiple producers: clone the transmitter, one per thread.
    let (tx, rx) = mpsc::channel();
    for id in 1..=3 {
        let tx = tx.clone();
        thread::spawn(move || {
            for job in 1..=2 {
                tx.send(format!("worker {id}, job {job}")).unwrap();
                thread::sleep(Duration::from_millis(2));
            }
        });
    }
    // The ORIGINAL `tx` is still alive here. Drop it, or `for msg in rx`
    // would wait forever for a message from it.
    drop(tx);
    let messages: Vec<String> = rx.iter().collect();
    println!(
        "3. {} messages from 3 producers: {messages:?}",
        messages.len()
    );

    // 4. `try_recv` never blocks: handy when main has other work to do.
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        thread::sleep(Duration::from_millis(20));
        tx.send("the report is ready").unwrap();
    });
    let mut checks = 0;
    let report = loop {
        match rx.try_recv() {
            Ok(message) => break message,
            Err(mpsc::TryRecvError::Empty) => {
                checks += 1; // do some other work, then look again
                thread::sleep(Duration::from_millis(2));
            }
            Err(mpsc::TryRecvError::Disconnected) => panic!("sender vanished"),
        }
    };
    handle.join().unwrap();
    println!("4. after {checks} checks: {report}");

    // 5. A BOUNDED channel: `sync_channel(2)` holds at most 2 messages, so a
    // fast sender has to wait for a slow receiver (back-pressure).
    let (tx, rx) = mpsc::sync_channel(2);
    let producer = thread::spawn(move || {
        for n in 1..=5 {
            tx.send(n).unwrap(); // blocks while 2 messages are waiting
            println!("5. sent {n}");
        }
    });
    for n in rx {
        thread::sleep(Duration::from_millis(3)); // a slow consumer
        println!("5.          received {n}");
    }
    producer.join().unwrap();

    // 6. Sending to a receiver that's gone returns Err, it doesn't panic.
    let (tx, rx) = mpsc::channel::<i32>();
    drop(rx);
    println!("6. send after the receiver dropped: {:?}", tx.send(1));
}
