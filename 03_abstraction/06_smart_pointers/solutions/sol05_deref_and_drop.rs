//! Solution to exercise 5: your own smart pointer, with `Deref`,
//! `DerefMut` and `Drop`.
//!
//! `deref` and `deref_mut` return a reference to the field inside. That is
//! all `*tracked` needs: Rust turns `*tracked` into `*(tracked.deref())`.
//! `drop` borrows the shared log mutably for one statement and pushes.
//!
//! Test: cargo test -p smart_pointers --features solutions --bin sol05_deref_and_drop

use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

type Log = Rc<RefCell<Vec<String>>>;

struct Tracked<T> {
    name: &'static str,
    value: T,
    log: Log,
}

impl<T> Tracked<T> {
    fn new(name: &'static str, value: T, log: &Log) -> Tracked<T> {
        Tracked {
            name,
            value,
            log: Rc::clone(log),
        }
    }
}

impl<T> Deref for Tracked<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> DerefMut for Tracked<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T> Drop for Tracked<T> {
    fn drop(&mut self) {
        self.log.borrow_mut().push(format!("dropped {}", self.name));
    }
}

fn shout(text: &str) -> String {
    text.to_uppercase()
}

fn main() {
    let log = Log::default();
    {
        let mut greeting = Tracked::new("greeting", String::from("hello"), &log);
        greeting.push_str(", world"); // DerefMut: a String method
        println!("{}", shout(&greeting)); // &Tracked<String> -> &String -> &str
        let _number = Tracked::new("number", 42, &log);
    } // both are dropped here, in reverse order
    println!("{:?}", log.borrow());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deref_reaches_the_value_inside() {
        let log = Log::default();
        let number = Tracked::new("number", 5, &log);
        assert_eq!(*number + 1, 6);
        let word = Tracked::new("word", String::from("hi"), &log);
        assert_eq!(word.len(), 2); // a String method, found through Deref
        assert_eq!(shout(&word), "HI"); // deref coercion, twice
    }

    #[test]
    fn deref_mut_changes_the_value_inside() {
        let log = Log::default();
        let mut list = Tracked::new("list", vec![1], &log);
        list.push(2);
        assert_eq!(*list, [1, 2]);
        *list = vec![9];
        assert_eq!(*list, [9]);
    }

    #[test]
    fn values_are_dropped_in_reverse_order() {
        let log = Log::default();
        {
            let _first = Tracked::new("first", (), &log);
            let _second = Tracked::new("second", (), &log);
        }
        assert_eq!(*log.borrow(), ["dropped second", "dropped first"]);
    }

    #[test]
    fn std_mem_drop_ends_a_value_early() {
        let log = Log::default();
        let early = Tracked::new("early", (), &log);
        let late = Tracked::new("late", (), &log);
        drop(early);
        assert_eq!(*log.borrow(), ["dropped early"]);
        drop(late);
        assert_eq!(*log.borrow(), ["dropped early", "dropped late"]);
    }
}
