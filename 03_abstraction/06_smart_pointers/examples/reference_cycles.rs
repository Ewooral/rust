//! Book 15.6: reference cycles leak memory; `Weak<T>` breaks them.
//!
//! Run: cargo run -p smart_pointers --example reference_cycles

use std::cell::RefCell;
use std::rc::{Rc, Weak};

// Part 1: a list whose tail can be changed, so we can make a cycle.
#[derive(Debug)]
enum List {
    Cons(i32, RefCell<Rc<List>>),
    Nil,
}

use List::{Cons, Nil};

impl List {
    fn head(&self) -> Option<i32> {
        match self {
            Cons(value, _) => Some(*value),
            Nil => None,
        }
    }

    fn tail(&self) -> Option<&RefCell<Rc<List>>> {
        match self {
            Cons(_, item) => Some(item),
            Nil => None,
        }
    }
}

/// Prints when it is freed, so we can SEE whether it ever is.
struct Noisy(&'static str);

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("  (freed {})", self.0);
    }
}

// Part 2: a tree. Parents OWN children; children only POINT to parents.
#[derive(Debug)]
struct Node {
    value: i32,
    parent: RefCell<Weak<Node>>,
    children: RefCell<Vec<Rc<Node>>>,
}

fn main() {
    println!("== a cycle: a -> b -> a");
    let a = Rc::new(Cons(5, RefCell::new(Rc::new(Nil))));
    let b = Rc::new(Cons(10, RefCell::new(Rc::clone(&a))));
    if let Some(link) = a.tail() {
        *link.borrow_mut() = Rc::clone(&b); // now a points at b, and b at a
    }
    println!(
        "a count = {}, b count = {}",
        Rc::strong_count(&a),
        Rc::strong_count(&b)
    );
    let after_a = a.tail().and_then(|link| link.borrow().head());
    println!(
        "a = {:?}, and the item after a is {:?} (that's b)",
        a.head(),
        after_a
    );
    // Printing `a` with {:?} would follow a -> b -> a -> ... forever and
    // overflow the stack.
    drop(a);
    drop(b);
    println!("a and b dropped: each count fell to 1, never 0. Leaked.\n");

    // The same leak, made visible with Drop: two Noisy values in a cycle.
    struct Pair {
        _name: Noisy,
        other: RefCell<Option<Rc<Pair>>>,
    }
    {
        let x = Rc::new(Pair {
            _name: Noisy("x"),
            other: RefCell::new(None),
        });
        let y = Rc::new(Pair {
            _name: Noisy("y"),
            other: RefCell::new(Some(Rc::clone(&x))),
        });
        *x.other.borrow_mut() = Some(Rc::clone(&y));
        println!("x and y point at each other; leaving their scope...");
    }
    println!("...and nothing was freed (no \"(freed ...)\" lines above).\n");

    println!("== a tree with Weak parent links");
    let leaf = Rc::new(Node {
        value: 3,
        parent: RefCell::new(Weak::new()),
        children: RefCell::new(vec![]),
    });
    println!(
        "leaf parent = {:?}",
        leaf.parent.borrow().upgrade().map(|p| p.value)
    );
    println!(
        "leaf strong = {}, weak = {}",
        Rc::strong_count(&leaf),
        Rc::weak_count(&leaf)
    );

    {
        let branch = Rc::new(Node {
            value: 5,
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(vec![Rc::clone(&leaf)]),
        });
        *leaf.parent.borrow_mut() = Rc::downgrade(&branch);

        println!(
            "leaf parent = {:?}",
            leaf.parent.borrow().upgrade().map(|p| p.value)
        );
        println!("branch has {} child", branch.children.borrow().len());
        println!(
            "branch strong = {}, weak = {}",
            Rc::strong_count(&branch),
            Rc::weak_count(&branch)
        );
        println!(
            "leaf strong = {}, weak = {}",
            Rc::strong_count(&leaf),
            Rc::weak_count(&leaf)
        );
    } // branch's strong count hits 0: it is freed, even though leaf points at it

    println!(
        "after branch's scope: leaf parent = {:?}",
        leaf.parent.borrow().upgrade().map(|p| p.value)
    );
    println!(
        "leaf strong = {}, weak = {}",
        Rc::strong_count(&leaf),
        Rc::weak_count(&leaf)
    );
}
