//! Book 15.5: `RefCell<T>` and interior mutability.
//!
//! Run:   cargo run -p smart_pointers --example refcell_mock_messenger
//! Test:  cargo test -p smart_pointers --example refcell_mock_messenger
//! Panic: cargo run -p smart_pointers --example refcell_mock_messenger -- panic
//!
//! A `LimitTracker` warns a user when they get close to a quota. It sends
//! messages through any `Messenger`, whose `send` takes `&self`. To test
//! it, a mock messenger must REMEMBER what it was sent, which means
//! changing itself through `&self`. RefCell makes that possible.

use std::cell::RefCell;

pub trait Messenger {
    fn send(&self, msg: &str);
}

pub struct LimitTracker<'a, T: Messenger> {
    messenger: &'a T,
    value: usize,
    max: usize,
}

impl<'a, T> LimitTracker<'a, T>
where
    T: Messenger,
{
    pub fn new(messenger: &'a T, max: usize) -> LimitTracker<'a, T> {
        LimitTracker {
            messenger,
            value: 0,
            max,
        }
    }

    pub fn set_value(&mut self, value: usize) {
        self.value = value;
        let percentage_of_max = self.value as f64 / self.max as f64;
        if percentage_of_max >= 1.0 {
            self.messenger.send("Error: You are over your quota!");
        } else if percentage_of_max >= 0.9 {
            self.messenger
                .send("Urgent warning: You've used up over 90% of your quota!");
        } else if percentage_of_max >= 0.75 {
            self.messenger
                .send("Warning: You've used up over 75% of your quota!");
        }
    }
}

/// The mock: it records messages instead of sending them. With a plain
/// `Vec`, `send(&self)` could not push (error E0596). Inside a RefCell, it can.
struct MockMessenger {
    sent_messages: RefCell<Vec<String>>,
}

impl MockMessenger {
    fn new() -> MockMessenger {
        MockMessenger {
            sent_messages: RefCell::new(vec![]),
        }
    }
}

impl Messenger for MockMessenger {
    fn send(&self, message: &str) {
        self.sent_messages.borrow_mut().push(String::from(message));
    }
}

fn main() {
    let messenger = MockMessenger::new();
    let mut tracker = LimitTracker::new(&messenger, 100);
    for value in [50, 80, 95, 120] {
        tracker.set_value(value);
    }
    for message in messenger.sent_messages.borrow().iter() {
        println!("sent: {message}");
    }

    // The borrowing rules still hold, but they are checked at RUN time:
    // any number of `borrow()`s OR one `borrow_mut()`, never both at once.
    let cell = RefCell::new(vec![1]);
    let reader = cell.borrow();
    println!("\nwhile a borrow() is alive:");
    println!("  try_borrow()     -> ok? {}", cell.try_borrow().is_ok());
    println!(
        "  try_borrow_mut() -> ok? {}",
        cell.try_borrow_mut().is_ok()
    );
    drop(reader);
    println!("after it ends:");
    println!(
        "  try_borrow_mut() -> ok? {}",
        cell.try_borrow_mut().is_ok()
    );

    if std::env::args().any(|a| a == "panic") {
        // The Book's broken mock does this inside `send`: two borrow_mut()s
        // at once. It compiles, and panics when it runs.
        let mut one_borrow = cell.borrow_mut();
        let mut two_borrow = cell.borrow_mut(); // panics: already borrowed
        one_borrow.push(2);
        two_borrow.push(3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_sends_an_over_75_percent_warning_message() {
        let mock_messenger = MockMessenger::new();
        let mut limit_tracker = LimitTracker::new(&mock_messenger, 100);

        limit_tracker.set_value(80);

        assert_eq!(mock_messenger.sent_messages.borrow().len(), 1);
    }

    #[test]
    fn nothing_is_sent_below_75_percent() {
        let mock_messenger = MockMessenger::new();
        let mut limit_tracker = LimitTracker::new(&mock_messenger, 100);

        limit_tracker.set_value(74);

        assert!(mock_messenger.sent_messages.borrow().is_empty());
    }
}
