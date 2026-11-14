//! Book 15.4: `Rc<T>`, the reference-counted smart pointer.
//!
//! Run: cargo run -p smart_pointers --example rc_shared_list
//!
//! Two lists share one tail:
//!
//!     b: 3 ─┐
//!           ├─> a: 5 -> 10 -> Nil
//!     c: 4 ─┘

use std::rc::Rc;

#[derive(Debug)]
enum List {
    Cons(i32, Rc<List>),
    Nil,
}

use List::{Cons, Nil};

fn total(list: &List) -> i32 {
    match list {
        Cons(value, rest) => value + total(rest), // &Rc<List> coerces to &List
        Nil => 0,
    }
}

fn main() {
    // With Box, `a` can have only one owner:
    //
    //     let b = Cons(3, Box::new(a));
    //     let c = Cons(4, Box::new(a));   // error[E0382]: use of moved value: `a`
    //
    // With Rc, each list holds its own pointer to the SAME `a`.
    let a = Rc::new(Cons(5, Rc::new(Cons(10, Rc::new(Nil)))));
    println!("count after creating a = {}", Rc::strong_count(&a));

    // Rc::clone doesn't copy the list: it adds 1 to the count. (Writing
    // `Rc::clone(&a)` rather than `a.clone()` shows the reader that.)
    let b = Cons(3, Rc::clone(&a));
    println!("count after creating b = {}", Rc::strong_count(&a));
    {
        let c = Cons(4, Rc::clone(&a));
        println!("count after creating c = {}", Rc::strong_count(&a));
        println!(
            "  totals: b = {}, c = {} (both include a's 5 + 10)",
            total(&b),
            total(&c)
        );
    }
    println!("count after c goes out of scope = {}", Rc::strong_count(&a));
    println!("a = {a:?}");

    // Rc gives SHARED (read-only) access. Changing the data would need
    // &mut, which Rc never hands out while it is shared:
    //
    //     let shared = Rc::new(vec![1, 2]);
    //     shared.push(3);   // error[E0596]: cannot borrow data in an `Rc` as mutable
    //
    // Section 15.5 adds RefCell for that.
    let shared = Rc::new(vec![1, 2]);
    let other = Rc::clone(&shared);
    println!("\nboth see {other:?}, {} owners", Rc::strong_count(&shared));

    // Rc::ptr_eq: are two Rcs pointing at the same value?
    println!("same vec? {}", Rc::ptr_eq(&shared, &other));
}
