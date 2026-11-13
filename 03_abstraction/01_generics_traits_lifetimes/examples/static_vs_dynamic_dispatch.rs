//! Beyond the Book: static dispatch (generics) vs dynamic dispatch (dyn Trait),
//! `impl Trait` in argument and return position, and dyn compatibility.
//! Run: cargo run -p generics_traits_lifetimes --example static_vs_dynamic_dispatch

use std::f64::consts::PI;

trait Shape {
    fn area(&self) -> f64;
    fn name(&self) -> String {
        String::from("shape")
    }
}

struct Circle(f64);
struct Square(f64);

impl Shape for Circle {
    fn area(&self) -> f64 {
        PI * self.0 * self.0
    }
    fn name(&self) -> String {
        format!("circle r={}", self.0)
    }
}

impl Shape for Square {
    fn area(&self) -> f64 {
        self.0 * self.0
    }
}

// ---- STATIC dispatch: generics ----
// Monomorphized: one copy per concrete type, direct calls, can be inlined.
// But a Vec<T> holds only ONE type T.
fn total_area_static<T: Shape>(shapes: &[T]) -> f64 {
    shapes.iter().map(|s| s.area()).sum()
}

// ---- DYNAMIC dispatch: trait objects ----
// `dyn Shape` = "some type implementing Shape, decided at run time". It must
// sit behind a pointer (&dyn, Box<dyn>, Rc<dyn>...). Each call looks up the
// method in a vtable. One compiled copy, mixed types in one collection.
fn total_area_dynamic(shapes: &[Box<dyn Shape>]) -> f64 {
    shapes.iter().map(|s| s.area()).sum()
}

// ---- impl Trait in ARGUMENT position: sugar for a generic (static) ----
fn describe(shape: &impl Shape) -> String {
    format!("{} has area {:.2}", shape.name(), shape.area())
}

// ---- impl Trait in RETURN position: one hidden concrete type (static) ----
fn unit_square() -> impl Shape {
    Square(1.0)
}

// To return DIFFERENT types depending on a value, you need dyn:
fn make_shape(round: bool) -> Box<dyn Shape> {
    if round {
        Box::new(Circle(1.0))
    } else {
        Box::new(Square(2.0))
    }
}
// fn make_shape_impl(round: bool) -> impl Shape {
//     if round { Circle(1.0) } else { Square(2.0) }
// }   // E0308: `if` and `else` have incompatible types

// ---- dyn compatibility (formerly "object safety") ----
// A trait can be used as `dyn Trait` only if the vtable can describe it.
// A generic method can't go in a vtable (infinitely many copies), and
// returning `Self` needs the concrete size. This trait is NOT dyn compatible:
trait Cloner {
    fn clone_me(&self) -> Self
    where
        Self: Sized; // `where Self: Sized` excludes this method from dyn use...
    fn label(&self) -> String; // ...so the trait stays usable as dyn for the rest
}
impl Cloner for Square {
    fn clone_me(&self) -> Self {
        Square(self.0)
    }
    fn label(&self) -> String {
        format!("square {}", self.0)
    }
}

fn main() {
    let circles = [Circle(1.0), Circle(2.0)];
    println!("static total: {:.2}", total_area_static(&circles));

    let mixed: Vec<Box<dyn Shape>> = vec![Box::new(Circle(1.0)), Box::new(Square(3.0))];
    println!("dynamic total: {:.2}", total_area_dynamic(&mixed));
    for s in &mixed {
        println!("  {}", s.name());
    }

    println!("{}", describe(&Square(2.0)));
    println!("unit square area: {}", unit_square().area());
    println!(
        "made: {}, {}",
        make_shape(true).name(),
        make_shape(false).name()
    );

    // A &dyn is a "fat pointer": data pointer + vtable pointer.
    println!(
        "&Square: {} bytes, &dyn Shape: {} bytes",
        size_of::<&Square>(),
        size_of::<&dyn Shape>()
    );

    let c: &dyn Cloner = &Square(5.0);
    println!("{}", c.label());
    let copy = Square(5.0).clone_me();
    println!("cloned area {}", copy.area());
}
