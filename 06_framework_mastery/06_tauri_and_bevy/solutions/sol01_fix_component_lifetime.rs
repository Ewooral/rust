//! Solution to exercise 1: make it compile (E0310).
//!
//! The heart of an ECS: one storage per component TYPE, all kept in one
//! map. The map's key is the type's `TypeId`, and the storages are boxed
//! as `dyn Any` so different `Vec<T>`s fit in one `HashMap`.
//!
//! `TypeId::of::<T>()` and `Box<dyn Any>` need `T: 'static`: the type may
//! not contain references that could expire. Both methods get the bound.
//!
//! Answer to the question: no. A `Label<'a>(&'a str)` borrowing a local
//! `String` is not `'static`, so `push` refuses it. That's exactly what
//! keeps an ECS safe: the world can keep components for as long as it
//! likes, so components must own their data (`String`, not `&str`), or
//! borrow only things that live forever (`&'static str`).
//!
//! Run: cargo run -p tauri_and_bevy --features solutions --bin sol01_fix_component_lifetime

use std::any::{Any, TypeId};
use std::collections::HashMap;

#[derive(Default)]
struct Storages {
    map: HashMap<TypeId, Box<dyn Any>>,
}

impl Storages {
    /// Adds a component to the storage for its type.
    fn push<T: 'static>(&mut self, component: T) {
        let storage = self
            .map
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(Vec::<T>::new()));
        storage
            .downcast_mut::<Vec<T>>()
            .expect("the storage for T is a Vec<T>")
            .push(component);
    }

    /// Every component of type `T`.
    fn all<T: 'static>(&self) -> &[T] {
        match self.map.get(&TypeId::of::<T>()) {
            Some(storage) => storage.downcast_ref::<Vec<T>>().expect("a Vec<T>"),
            None => &[],
        }
    }
}

#[derive(Debug)]
struct Position(i32, i32);
#[derive(Debug)]
struct Name(&'static str);

fn main() {
    let mut storages = Storages::default();
    storages.push(Position(1, 2));
    storages.push(Name("Rusty"));
    storages.push(Position(5, 5));
    println!("positions: {:?}", storages.all::<Position>());
    println!("names:     {:?}", storages.all::<Name>());
    println!("speeds:    {:?}", storages.all::<f32>());
    let sum: i32 = storages.all::<Position>().iter().map(|p| p.0 + p.1).sum();
    println!(
        "sum of all coordinates: {sum}, first name: {}",
        storages.all::<Name>()[0].0
    );
}
