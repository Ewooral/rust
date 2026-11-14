//! Exercise 6: a family tree with `Weak` parent links (Book 15.6).
//!
//! Each `Person` owns their children (`Rc`) and points to their parent with
//! a `Weak`, so a parent and child never keep each other alive. Implement:
//! - `add_child(parent, name)`: make a person whose parent link points at
//!   `parent` (`Rc::downgrade`), add them to the parent's children, and
//!   return them;
//! - `parent_name`: the parent's name, if the parent still exists
//!   (`upgrade` the `Weak`);
//! - `ancestors`: the names from the parent up to the oldest ancestor;
//! - `descendant_count`: children, grandchildren, and so on, all counted.
//!
//! The tests are written; make them pass. The last one checks for a leak.
//!
//! Test: cargo test -p smart_pointers --features exercises --bin ex06_weak_tree

#![allow(unused_variables, dead_code)]

use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Debug)]
struct Person {
    name: String,
    parent: RefCell<Weak<Person>>,
    children: RefCell<Vec<Rc<Person>>>,
}

impl Person {
    fn new(name: &str) -> Rc<Person> {
        Rc::new(Person {
            name: name.to_string(),
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(Vec::new()),
        })
    }

    fn add_child(parent: &Rc<Person>, name: &str) -> Rc<Person> {
        todo!()
    }

    fn parent_name(&self) -> Option<String> {
        todo!()
    }

    fn ancestors(&self) -> Vec<String> {
        todo!()
    }

    fn descendant_count(&self) -> usize {
        todo!()
    }
}

fn main() {
    let grandma = Person::new("Abena");
    let mum = Person::add_child(&grandma, "Efua");
    let me = Person::add_child(&mum, "Kojo");
    Person::add_child(&mum, "Esi");
    println!("{}'s parent: {:?}", me.name, me.parent_name());
    println!("{}'s ancestors: {:?}", me.name, me.ancestors());
    println!(
        "{} has {} descendants",
        grandma.name,
        grandma.descendant_count()
    );
    println!(
        "{}: strong = {}, weak = {}",
        mum.name,
        Rc::strong_count(&mum),
        Rc::weak_count(&mum)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn family() -> (Rc<Person>, Rc<Person>, Rc<Person>) {
        let grandma = Person::new("Abena");
        let mum = Person::add_child(&grandma, "Efua");
        let me = Person::add_child(&mum, "Kojo");
        Person::add_child(&mum, "Esi");
        (grandma, mum, me)
    }

    #[test]
    fn children_know_their_parent() {
        let (grandma, mum, me) = family();
        assert_eq!(me.parent_name().as_deref(), Some("Efua"));
        assert_eq!(mum.parent_name().as_deref(), Some("Abena"));
        assert_eq!(grandma.parent_name(), None);
        assert_eq!(mum.children.borrow().len(), 2);
    }

    #[test]
    fn ancestors_and_descendants() {
        let (grandma, mum, me) = family();
        assert_eq!(me.ancestors(), ["Efua", "Abena"]);
        assert!(grandma.ancestors().is_empty());
        assert_eq!(grandma.descendant_count(), 3);
        assert_eq!(mum.descendant_count(), 2);
        assert_eq!(me.descendant_count(), 0);
    }

    #[test]
    fn weak_links_do_not_own_so_nothing_leaks() {
        let (grandma, mum, _me) = family();
        // Owners of mum: the `mum` variable and grandma's children list.
        assert_eq!(Rc::strong_count(&mum), 2);
        // Weak links to mum: the parent links of her two children.
        assert_eq!(Rc::weak_count(&mum), 2);

        let watch = Rc::downgrade(&grandma);
        drop(grandma); // mum still points up at her, but only weakly
        assert!(watch.upgrade().is_none(), "grandma was freed");
        assert_eq!(mum.parent_name(), None);
        assert_eq!(Rc::strong_count(&mum), 1, "grandma's list let go of mum");
    }
}
