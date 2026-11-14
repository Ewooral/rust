//! Solution to exercise 6: a family tree with `Weak` parent links.
//!
//! `add_child` stores `Rc::downgrade(parent)` in the child (a link that
//! doesn't own) and an `Rc` of the child in the parent (a link that does).
//! Walking up uses `upgrade()`, which gives `None` once a person is gone.
//!
//! Test: cargo test -p smart_pointers --features solutions --bin sol06_weak_tree

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
        let child = Person::new(name);
        *child.parent.borrow_mut() = Rc::downgrade(parent);
        parent.children.borrow_mut().push(Rc::clone(&child));
        child
    }

    fn parent_name(&self) -> Option<String> {
        let parent = self.parent.borrow().upgrade()?;
        Some(parent.name.clone())
    }

    fn ancestors(&self) -> Vec<String> {
        let mut names = Vec::new();
        let mut next = self.parent.borrow().upgrade();
        while let Some(person) = next {
            names.push(person.name.clone());
            next = person.parent.borrow().upgrade();
        }
        names
    }

    fn descendant_count(&self) -> usize {
        self.children
            .borrow()
            .iter()
            .map(|child| 1 + child.descendant_count())
            .sum()
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
