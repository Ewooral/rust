//! Book 15.5: `Rc<RefCell<T>>`: several owners who can ALL change the value.
//!
//! Run: cargo run -p smart_pointers --example rc_refcell_shared

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
enum List {
    Cons(Rc<RefCell<i32>>, Rc<List>),
    Nil,
}

use List::{Cons, Nil};

/// Reads every value in a list (a shared borrow of each RefCell).
fn values(list: &List) -> Vec<i32> {
    match list {
        Cons(value, rest) => {
            let mut all = vec![*value.borrow()];
            all.extend(values(rest));
            all
        }
        Nil => vec![],
    }
}

fn main() {
    // `value` is shared (Rc) and changeable (RefCell).
    let value = Rc::new(RefCell::new(5));

    let a = Rc::new(Cons(Rc::clone(&value), Rc::new(Nil)));
    let b = Cons(Rc::new(RefCell::new(3)), Rc::clone(&a));
    let c = Cons(Rc::new(RefCell::new(4)), Rc::clone(&a));

    // Change it through ONE owner...
    *value.borrow_mut() += 10;

    // ...and every list that shares it sees the change.
    println!("a after = {a:?}");
    println!("b after = {b:?}");
    println!("c after = {c:?}");
    println!("values in b: {:?}", values(&b));

    // A real-world shape: a shared to-do list that two "screens" both edit.
    let todo: Rc<RefCell<Vec<&str>>> = Rc::new(RefCell::new(Vec::new()));
    let phone = Rc::clone(&todo);
    let laptop = Rc::clone(&todo);
    phone.borrow_mut().push("buy milk");
    laptop.borrow_mut().push("walk the dog");
    println!("\nshared to-do list: {:?}", todo.borrow());
    println!("owners: {}", Rc::strong_count(&todo));
}

// Choosing a pointer:
//   Box<T>         one owner;  borrow checked at compile time
//   Rc<T>          many owners; read-only
//   RefCell<T>     one owner;  mutable through &self, checked at run time
//   Rc<RefCell<T>> many owners, all may change it
// Rc and RefCell are for ONE thread. Across threads: Arc and Mutex (chapter 16).
