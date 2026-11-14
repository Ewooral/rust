//! Exercise 4: a sparse-set component storage.
//!
//! The lesson's `Storage<T>` is a `Vec<Option<T>>` indexed by entity: simple,
//! but a component that only 3 of 10,000 entities have still costs 10,000
//! slots, and iterating means skipping 9,997 `None`s. Bevy offers a second
//! kind of storage for such components, the SPARSE SET:
//!
//! - `dense`: the components, packed together with no gaps,
//! - `entities`: which entity each dense slot belongs to,
//! - `sparse`: for each entity index, where its component is in `dense`.
//!
//! Implement `insert` (replacing returns the old value), `get`, `get_mut`
//! and `remove`. (`iter` is done for you: with no gaps, it's one `zip`.)
//! `remove` must keep `dense` packed: use
//! `Vec::swap_remove` (the last element moves into the hole), then fix the
//! `sparse` entry of the entity that moved.
//!
//! Test: cargo test -p tauri_and_bevy --features exercises --bin ex04_sparse_set

#![allow(unused_variables, dead_code)]

#[derive(Debug, Default)]
struct SparseSet<T> {
    sparse: Vec<Option<usize>>,
    dense: Vec<T>,
    entities: Vec<usize>,
}

impl<T> SparseSet<T> {
    fn new() -> Self {
        SparseSet {
            sparse: Vec::new(),
            dense: Vec::new(),
            entities: Vec::new(),
        }
    }

    fn len(&self) -> usize {
        self.dense.len()
    }

    /// Adds or replaces `entity`'s component; returns the old one.
    fn insert(&mut self, entity: usize, value: T) -> Option<T> {
        todo!()
    }

    fn get(&self, entity: usize) -> Option<&T> {
        todo!()
    }

    fn get_mut(&mut self, entity: usize) -> Option<&mut T> {
        todo!()
    }

    /// Removes `entity`'s component, keeping `dense` packed.
    fn remove(&mut self, entity: usize) -> Option<T> {
        todo!()
    }

    /// Every (entity, component), in dense order: no gaps to skip.
    fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        self.entities.iter().copied().zip(self.dense.iter())
    }
}

fn main() {
    let mut poisoned: SparseSet<&str> = SparseSet::new();
    poisoned.insert(9_000, "spider bite");
    poisoned.insert(3, "bad berries");
    poisoned.insert(42, "snake");
    poisoned.remove(3);
    if let Some(cause) = poisoned.get_mut(42) {
        *cause = "snake (treated)";
    }
    println!("entity 42: {:?}", poisoned.get(42));
    println!(
        "{} poisoned, sparse table has {} slots",
        poisoned.len(),
        poisoned.sparse.len()
    );
    for (entity, cause) in poisoned.iter() {
        println!("entity {entity}: {cause}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_get_and_replace() {
        let mut set = SparseSet::new();
        assert_eq!(set.insert(5, "a"), None);
        assert_eq!(set.insert(2, "b"), None);
        assert_eq!(set.get(5), Some(&"a"));
        assert_eq!(set.get(2), Some(&"b"));
        assert_eq!(set.get(3), None);
        assert_eq!(set.get(100), None);
        assert_eq!(set.insert(5, "c"), Some("a"));
        assert_eq!(set.len(), 2);
        *set.get_mut(2).unwrap() = "d";
        assert_eq!(set.get(2), Some(&"d"));
    }

    #[test]
    fn remove_keeps_everything_else_reachable() {
        let mut set = SparseSet::new();
        for entity in [10, 20, 30, 40] {
            set.insert(entity, entity * 2);
        }
        assert_eq!(set.remove(20), Some(40));
        assert_eq!(set.remove(20), None, "already removed");
        assert_eq!(set.remove(7), None, "never there");
        assert_eq!(set.len(), 3);
        for entity in [10, 30, 40] {
            assert_eq!(set.get(entity), Some(&(entity * 2)), "entity {entity}");
        }
        assert_eq!(set.get(20), None);
    }

    #[test]
    fn removing_the_last_one_and_reinserting() {
        let mut set = SparseSet::new();
        set.insert(1, 'x');
        set.insert(2, 'y');
        assert_eq!(set.remove(2), Some('y'));
        assert_eq!(set.remove(1), Some('x'));
        assert_eq!(set.len(), 0);
        set.insert(2, 'z');
        assert_eq!(set.get(2), Some(&'z'));
    }

    #[test]
    fn iter_has_no_gaps() {
        let mut set = SparseSet::new();
        set.insert(1000, "far");
        set.insert(0, "near");
        set.insert(500, "middle");
        set.remove(0);
        let mut seen: Vec<_> = set.iter().collect();
        seen.sort();
        assert_eq!(seen, [(500, &"middle"), (1000, &"far")]);
    }
}
