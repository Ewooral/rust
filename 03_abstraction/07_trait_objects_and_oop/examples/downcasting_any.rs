//! Beyond the Book: downcasting with `std::any::Any`, getting the concrete
//! type back out of a trait object.
//! Run: cargo run -p trait_objects_and_oop --example downcasting_any
//!
//! A `dyn Trait` forgets the concrete type on purpose. Occasionally you
//! really need it back ("is this widget a Slider? then set its value").
//! `Any` keeps a hidden TypeId for every `'static` type, so you can ask:
//!
//!   any.is::<T>()            -> bool
//!   any.downcast_ref::<T>()  -> Option<&T>
//!   any.downcast_mut::<T>()  -> Option<&mut T>
//!   boxed.downcast::<T>()    -> Result<Box<T>, Box<dyn Any>>

use std::any::{Any, TypeId};
use std::fmt::Debug;

// ---------- 1. Plain `dyn Any` ----------
fn describe(value: &dyn Any) -> String {
    if let Some(n) = value.downcast_ref::<i32>() {
        format!("an i32: {n}")
    } else if let Some(s) = value.downcast_ref::<String>() {
        format!("a String of {} bytes", s.len())
    } else if value.is::<f64>() {
        "some f64".to_string()
    } else {
        "something else".to_string()
    }
}

// ---------- 2. Downcasting your own trait objects ----------
// Make `Any` a supertrait. Then a `&dyn Widget` can be UPCAST to `&dyn Any`
// (trait upcasting, stable since Rust 1.86) and downcast from there.
trait Widget: Any + Debug {
    fn name(&self) -> &str;
}

#[derive(Debug)]
struct Button {
    label: String,
}

#[derive(Debug)]
struct Slider {
    value: u8,
}

impl Widget for Button {
    fn name(&self) -> &str {
        &self.label
    }
}

impl Widget for Slider {
    fn name(&self) -> &str {
        "slider"
    }
}

/// Finds every Slider among mixed widgets and sets its value.
fn set_all_sliders(widgets: &mut [Box<dyn Widget>], value: u8) -> usize {
    let mut changed = 0;
    for widget in widgets.iter_mut() {
        let any: &mut dyn Any = widget.as_mut(); // upcast &mut dyn Widget -> &mut dyn Any
        if let Some(slider) = any.downcast_mut::<Slider>() {
            slider.value = value;
            changed += 1;
        }
    }
    changed
}

fn main() {
    let values: Vec<Box<dyn Any>> = vec![
        Box::new(42_i32),
        Box::new(String::from("hello")),
        Box::new(2.5_f64),
        Box::new('x'),
    ];
    for v in &values {
        // `v` is a &Box<dyn Any>: deref to the `dyn Any` INSIDE the box.
        println!("{}", describe(v.as_ref()));
    }

    // Taking ownership back out of a Box<dyn Any>:
    let boxed: Box<dyn Any> = Box::new(String::from("owned again"));
    match boxed.downcast::<String>() {
        Ok(s) => println!("got the String back: {s}"),
        Err(_) => println!("not a String"),
    }

    // A classic trap: call type_id on the CONTENTS, not on the Box. On a
    // `Box<dyn Any>`, `b.type_id()` would describe the Box itself (clippy
    // warns: `type_id_on_box`).
    let b: Box<dyn Any> = Box::new(7_u8);
    println!("box holds a u8? {}", (*b).type_id() == TypeId::of::<u8>());

    let mut widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Button { label: "OK".into() }),
        Box::new(Slider { value: 0 }),
        Box::new(Slider { value: 10 }),
    ];
    let changed = set_all_sliders(&mut widgets, 99);
    println!("changed {changed} sliders: {widgets:?}");
    let names: Vec<&str> = widgets.iter().map(|w| w.name()).collect();
    println!("names: {names:?}");

    // `Any` needs 'static: a type holding a short borrow (like `&'a str`)
    // can't be `dyn Any`. And treat downcasting as a last resort: usually a
    // new trait method, or an enum, says the same thing more clearly.
}
