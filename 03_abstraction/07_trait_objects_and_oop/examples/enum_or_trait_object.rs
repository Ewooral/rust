//! Choosing between an ENUM and a TRAIT OBJECT for "a list of different
//! kinds of thing". Both work; they make opposite things easy.
//! Run: cargo run -p trait_objects_and_oop --example enum_or_trait_object
//!
//!                          enum Shape { .. }        Box<dyn Shape>
//!   the set of kinds is    closed (you own it)      open (anyone can add)
//!   add a new KIND         edit the enum + matches  just `impl Shape for New`
//!   add a new OPERATION    one new `match`          edit the trait + every impl
//!   compiler checks        every case is handled    every impl has the methods
//!   cost                   no Box, no vtable        a Box and a vtable call

use std::f64::consts::PI;

// ---------- Closed set: an enum ----------
enum ShapeEnum {
    Circle { radius: f64 },
    Square { side: f64 },
}

impl ShapeEnum {
    fn area(&self) -> f64 {
        match self {
            ShapeEnum::Circle { radius } => PI * radius * radius,
            ShapeEnum::Square { side } => side * side,
        }
    }
}

// ---------- Open set: a trait object ----------
trait Shape {
    fn area(&self) -> f64;
}

struct Circle {
    radius: f64,
}
struct Square {
    side: f64,
}
// A brand-new kind, added without touching anything above:
struct Triangle {
    base: f64,
    height: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        PI * self.radius * self.radius
    }
}
impl Shape for Square {
    fn area(&self) -> f64 {
        self.side * self.side
    }
}
impl Shape for Triangle {
    fn area(&self) -> f64 {
        0.5 * self.base * self.height
    }
}

fn main() {
    let closed = [
        ShapeEnum::Circle { radius: 1.0 },
        ShapeEnum::Square { side: 2.0 },
    ];
    let total: f64 = closed.iter().map(ShapeEnum::area).sum();
    println!("enum total area:  {total:.2}");

    let open: Vec<Box<dyn Shape>> = vec![
        Box::new(Circle { radius: 1.0 }),
        Box::new(Square { side: 2.0 }),
        Box::new(Triangle {
            base: 3.0,
            height: 2.0,
        }),
    ];
    let total: f64 = open.iter().map(|s| s.area()).sum();
    println!("dyn total area:   {total:.2}");

    // Rule of thumb: if YOU know every kind (states of a parcel, tokens of a
    // parser), use an enum. If OTHER code must be able to add kinds (plugins,
    // GUI widgets, the Book's `Draw`), use a trait object.
}
