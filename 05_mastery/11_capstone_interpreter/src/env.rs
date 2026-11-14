//! Environments: where variables live.
//!
//! Each block and each function call gets a new `Env` whose `parent` is the
//! scope around it. Looking a name up walks outwards until it's found.
//!
//! Scopes are shared and changed from several places at once: a closure
//! keeps its defining scope alive (several owners: `Rc`), and assigning to
//! a variable changes a scope that others can see (mutation through a
//! shared pointer: `RefCell`). Hence `Rc<RefCell<Env>>`, from lesson 03/06.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

/// One scope: its own variables, and the scope around it.
#[derive(Default)]
pub struct Env {
    values: HashMap<String, Value>,
    parent: Option<Rc<RefCell<Env>>>,
}

impl Env {
    /// The outermost scope.
    pub fn new_global() -> Rc<RefCell<Env>> {
        Rc::new(RefCell::new(Env::default()))
    }

    /// A new scope inside `parent`.
    pub fn child(parent: &Rc<RefCell<Env>>) -> Rc<RefCell<Env>> {
        Rc::new(RefCell::new(Env {
            values: HashMap::new(),
            parent: Some(Rc::clone(parent)),
        }))
    }

    /// Creates (or replaces) a variable in THIS scope: `let x = ...`.
    pub fn define(&mut self, name: impl Into<String>, value: Value) {
        self.values.insert(name.into(), value);
    }

    /// Looks a name up here, then in each enclosing scope.
    pub fn get(&self, name: &str) -> Option<Value> {
        match self.values.get(name) {
            Some(value) => Some(value.clone()),
            None => self.parent.as_ref()?.borrow().get(name),
        }
    }

    /// Changes an existing variable in the nearest scope that has it:
    /// `x = ...`. Returns false if no scope has it.
    pub fn assign(&mut self, name: &str, value: Value) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn num(n: f64) -> Value {
        Value::Number(n)
    }

    #[test]
    fn lookup_walks_outwards() {
        let global = Env::new_global();
        global.borrow_mut().define("x", num(1.0));
        let inner = Env::child(&global);
        assert_eq!(inner.borrow().get("x"), Some(num(1.0)));
        assert_eq!(inner.borrow().get("y"), None);
    }

    #[test]
    fn define_shadows_assign_changes_the_original() {
        let global = Env::new_global();
        global.borrow_mut().define("x", num(1.0));
        let inner = Env::child(&global);

        inner.borrow_mut().define("x", num(2.0)); // shadow: a new x
        assert_eq!(inner.borrow().get("x"), Some(num(2.0)));
        assert_eq!(global.borrow().get("x"), Some(num(1.0)));

        let other = Env::child(&global);
        assert!(other.borrow_mut().assign("x", num(3.0))); // changes the global x
        assert_eq!(global.borrow().get("x"), Some(num(3.0)));
        assert!(!other.borrow_mut().assign("nope", num(0.0)));
    }

    #[test]
    fn a_child_keeps_its_parent_alive() {
        let inner = {
            let global = Env::new_global();
            global.borrow_mut().define("kept", num(42.0));
            Env::child(&global)
        }; // `global` the variable is gone, but the Rc inside `inner` isn't
        assert_eq!(inner.borrow().get("kept"), Some(num(42.0)));
    }
}
