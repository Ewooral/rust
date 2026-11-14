//! Exercise 1: make it compile (E0038).
//!
//! `Shape` has a method that returns `Self`. Behind a `dyn Shape` nobody
//! knows what `Self` is (or how big it is), so the compiler can't build a
//! vtable and rejects `Box<dyn Shape>`. Keep the `scaled` method, because
//! `main` still uses it on a concrete `Circle`, but opt it OUT of the vtable.
//! Change only the trait, not the impls or `main`.
//!
//! Run: cargo run -p trait_objects_and_oop --features exercises --bin ex01_fix_not_dyn_compatible

use std::f64::consts::PI;

trait Shape {
    fn area(&self) -> f64;
    fn scaled(&self, factor: f64) -> Self;
}

struct Circle {
    radius: f64,
}

struct Square {
    side: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        PI * self.radius * self.radius
    }
    fn scaled(&self, factor: f64) -> Self {
        Circle {
            radius: self.radius * factor,
        }
    }
}

impl Shape for Square {
    fn area(&self) -> f64 {
        self.side * self.side
    }
    fn scaled(&self, factor: f64) -> Self {
        Square {
            side: self.side * factor,
        }
    }
}

fn main() {
    let shapes: Vec<Box<dyn Shape>> = vec![
        Box::new(Circle { radius: 1.0 }),
        Box::new(Square { side: 2.0 }),
    ];
    let total: f64 = shapes.iter().map(|s| s.area()).sum();
    println!("total area: {total:.2}");

    let big = Circle { radius: 1.0 }.scaled(3.0);
    println!("scaled circle area: {:.2}", big.area());
}
