//! Solution to exercise 2: make it compile (E0596), with `Rc<RefCell<...>>`.
//!
//! Scopes for an interpreter. A block's scope has a `parent` (the scope
//! around it), and several child scopes share one parent, so the parent is
//! behind an `Rc`. But `x = 10` inside a block must CHANGE the parent's
//! variable, and an `Rc` only ever hands out shared (`&`) references.
//!
//! `RefCell` moves the "one writer OR many readers" check from compile
//! time to run time: `.borrow()` gives a shared reference, `.borrow_mut()`
//! an exclusive one, and breaking the rule panics instead of failing to
//! compile. `Rc<RefCell<Env>>` = shared by many, changeable by any of them,
//! one at a time.
//!
//! Run: cargo run -p capstone_interpreter --features solutions --bin sol02_fix_shared_env

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Default)]
struct Env {
    values: HashMap<String, f64>,
    parent: Option<Rc<RefCell<Env>>>,
}

impl Env {
    fn define(&mut self, name: &str, value: f64) {
        self.values.insert(name.to_string(), value);
    }

    fn get(&self, name: &str) -> Option<f64> {
        match self.values.get(name) {
            Some(value) => Some(*value),
            None => self.parent.as_ref()?.borrow().get(name),
        }
    }

    /// Changes the variable in the nearest scope that has it.
    fn assign(&mut self, name: &str, value: f64) -> bool {
        if let Some(slot) = self.values.get_mut(name) {
            *slot = value;
            return true;
        }
        match &self.parent {
            Some(parent) => parent.borrow_mut().assign(name, value),
            None => false,
        }
    }
}

fn main() {
    let global = Rc::new(RefCell::new(Env::default()));
    global.borrow_mut().define("x", 1.0);

    // Two blocks inside the global scope share it as their parent.
    let mut block_a = Env {
        values: HashMap::new(),
        parent: Some(Rc::clone(&global)),
    };
    let block_b = Env {
        values: HashMap::new(),
        parent: Some(Rc::clone(&global)),
    };

    block_a.assign("x", 10.0); // `x = 10;` inside block A
    println!("block B sees x = {:?}", block_b.get("x"));
    println!("global: x = {:?}", global.borrow().get("x"));
}
