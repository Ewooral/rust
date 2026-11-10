//! Beyond the Book: moving a value out of a `&mut` place.
//! You cannot move out of a borrow, but you can swap something else in.
//! Run: cargo run -p ownership --example mem_take_replace_swap

use std::mem;

#[derive(Debug, Default)]
struct Inbox {
    messages: Vec<String>,
}

impl Inbox {
    /// Hand all messages to the caller and leave the inbox empty.
    fn drain(&mut self) -> Vec<String> {
        // Returning `self.messages` directly fails with E0507:
        // "cannot move out of `self.messages` which is behind a mutable reference".
        mem::take(&mut self.messages) // leaves Vec::default() behind
    }
}

fn main() {
    let mut inbox = Inbox {
        messages: vec!["hi".into(), "hey".into()],
    };
    let taken = inbox.drain();
    println!("taken = {taken:?}, inbox = {inbox:?}");

    let mut name = String::from("old");
    let previous = mem::replace(&mut name, String::from("new"));
    println!("previous = {previous}, name = {name}");

    let mut a = String::from("left");
    let mut b = String::from("right");
    mem::swap(&mut a, &mut b);
    println!("a = {a}, b = {b}");
}
