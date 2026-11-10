//! Exercise 6 (Beyond the Book): move values out of `&mut self`.
//!
//! Implement the `Queue` methods. You only have `&mut self`, so you cannot
//! move `self.items` out directly. Use `std::mem::take` and `std::mem::replace`.
//!
//! Test: cargo test -p ownership --features exercises --bin ex06_mem_take

#![allow(unused_variables, unused_imports)]

use std::mem;

#[derive(Debug, Default)]
struct Queue {
    items: Vec<String>,
}

impl Queue {
    fn push(&mut self, item: &str) {
        todo!()
    }

    /// Returns every item and leaves the queue empty.
    fn flush(&mut self) -> Vec<String> {
        todo!()
    }

    /// Replaces the first item and returns the old one, or `None` if empty.
    fn replace_first(&mut self, item: &str) -> Option<String> {
        todo!()
    }
}

fn main() {
    let mut q = Queue::default();
    q.push("a");
    q.push("b");
    println!("old first: {:?}", q.replace_first("z"));
    println!("flushed: {:?}, left: {:?}", q.flush(), q);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flush_returns_all_and_empties() {
        let mut q = Queue::default();
        q.push("a");
        q.push("b");
        assert_eq!(q.flush(), vec!["a", "b"]);
        assert!(q.items.is_empty());
        assert!(q.flush().is_empty());
    }

    #[test]
    fn replace_first_returns_old_value() {
        let mut q = Queue::default();
        assert_eq!(q.replace_first("x"), None);
        q.push("a");
        q.push("b");
        assert_eq!(q.replace_first("z"), Some("a".to_string()));
        assert_eq!(q.items, vec!["z", "b"]);
    }
}
