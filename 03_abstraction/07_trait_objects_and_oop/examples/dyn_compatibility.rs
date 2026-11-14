//! Beyond the Book: dyn compatibility (once called "object safety").
//! Run: cargo run -p trait_objects_and_oop --example dyn_compatibility
//!
//! `dyn Trait` works through a vtable: one fixed table of function pointers
//! per concrete type. A trait can be used as `dyn` only if such a table can
//! be built. Each method must either fit in the vtable, or be marked
//! `where Self: Sized` ("not for trait objects"). A method fits when:
//!
//! - it has a receiver: `&self`, `&mut self`, `self: Box<Self>` (or Rc, Arc, Pin)
//! - it has no generic TYPE parameters (lifetimes are fine)
//! - it doesn't mention `Self` except in the receiver (no `-> Self`)
//! - it doesn't return `impl Trait` and isn't an `async fn`
//!
//! And the trait itself must have no associated consts, must not require
//! `Self: Sized`, and its supertraits must be dyn compatible too.

use std::fmt::Debug;

// ---------- Breaking the rules: each of these is E0038 if used as dyn ----------
#[allow(dead_code)]
trait Logger {
    fn log<T: Debug>(&self, value: T); // generic method: one copy per T, no fixed slot
}
// let l: &dyn Logger = &MyLogger;
//   // E0038: the trait `Logger` is not dyn compatible
//   //        ...because method `log` has generic type parameters

#[allow(dead_code)]
trait Duplicate {
    fn duplicate(&self) -> Self; // what size is `Self` behind a dyn? Unknown.
}
// let d: Box<dyn Duplicate> = ...;
//   // E0038: ...because method `duplicate` references the `Self` type in its return type

// `Clone` itself requires `Self: Sized`, so `Box<dyn Clone>` is E0038 too.

// ---------- Fix 1: take a trait object instead of a generic ----------
trait DynLogger {
    fn log(&self, value: &dyn Debug); // one function that handles every type
}

struct StdoutLogger;

impl DynLogger for StdoutLogger {
    fn log(&self, value: &dyn Debug) {
        println!("log: {value:?}");
    }
}

// ---------- Fix 2: `where Self: Sized` opts a method out of the vtable ----------
trait Shape {
    fn area(&self) -> f64;

    // A constructor has no `self` at all...
    fn new_unit() -> Self
    where
        Self: Sized;

    // ...and this one returns `Self`. Both are fine once opted out.
    fn doubled(&self) -> Self
    where
        Self: Sized;

    // ---------- Fix 3: the "clone_box" pattern ----------
    // Return a Box<dyn Shape> instead of Self, and it fits in the vtable.
    fn clone_box(&self) -> Box<dyn Shape>;
}

#[derive(Clone, Debug)]
struct Square(f64);

impl Shape for Square {
    fn area(&self) -> f64 {
        self.0 * self.0
    }
    fn new_unit() -> Self {
        Square(1.0)
    }
    fn doubled(&self) -> Self {
        Square(self.0 * 2.0)
    }
    fn clone_box(&self) -> Box<dyn Shape> {
        Box::new(self.clone())
    }
}

// Now `Box<dyn Shape>` can be cloned, so `Vec<Box<dyn Shape>>` can too.
impl Clone for Box<dyn Shape> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

fn main() {
    let logger: &dyn DynLogger = &StdoutLogger;
    logger.log(&42);
    logger.log(&"a string");
    logger.log(&vec![1, 2, 3]);

    // The Sized-only methods still work on concrete types...
    let unit = Square::new_unit();
    println!("unit {:?}, doubled {:?}", unit, unit.doubled());

    // ...and the trait is still usable as dyn for everything else.
    let shapes: Vec<Box<dyn Shape>> = vec![Box::new(Square(2.0)), Box::new(Square(3.0))];
    let copies = shapes.clone(); // uses clone_box under the hood
    let total: f64 = copies.iter().map(|s| s.area()).sum();
    println!("cloned {} shapes, total area {total}", copies.len());
    // shapes[0].doubled();
    //   // error: the `doubled` method cannot be invoked on a trait object
}
