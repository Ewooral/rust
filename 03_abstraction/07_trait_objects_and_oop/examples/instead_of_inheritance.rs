//! Book 18.1: Rust has no inheritance. Here is what you use instead, for each
//! of the two reasons people reach for inheritance.
//! Run: cargo run -p trait_objects_and_oop --example instead_of_inheritance
//!
//! Reason 1, REUSE code       -> default trait methods, and composition
//!                               (a struct that HAS another struct inside).
//! Reason 2, POLYMORPHISM     -> traits: generics (static) or `dyn` (dynamic).

// ---------- Reuse with a default method ----------
// In an OO language, `Animal` might be a base class with `describe` written
// once. In Rust, the shared code is a default method on a trait.
trait Animal {
    fn name(&self) -> String;
    fn sound(&self) -> String;

    /// Written once, reused by every type that does not override it.
    fn describe(&self) -> String {
        format!("{} says {}", self.name(), self.sound())
    }
}

struct Dog;
struct Owl;

impl Animal for Dog {
    fn name(&self) -> String {
        "Dog".into()
    }
    fn sound(&self) -> String {
        "woof".into()
    }
    // keeps the default `describe`
}

impl Animal for Owl {
    fn name(&self) -> String {
        "Owl".into()
    }
    fn sound(&self) -> String {
        "hoo".into()
    }
    // "overrides" the default, like overriding a method in a subclass
    fn describe(&self) -> String {
        format!("{} says {}, but only at night", self.name(), self.sound())
    }
}

// ---------- Reuse with composition ----------
// Instead of `class Car extends Engine`, a Car HAS an Engine and passes work
// to it ("delegation"). You choose exactly what to expose.
struct Engine {
    horsepower: u32,
}

impl Engine {
    fn start(&self) -> String {
        format!("vroom ({} hp)", self.horsepower)
    }
}

struct Car {
    name: String,
    engine: Engine, // composition: a part, not a parent
}

impl Car {
    fn start(&self) -> String {
        format!("{}: {}", self.name, self.engine.start()) // delegation
    }
}

// ---------- Polymorphism without inheritance ----------
// Generic: "any T with the Animal badge", decided at compile time.
// The Book calls this *bounded parametric polymorphism*.
fn introduce<T: Animal>(animal: &T) -> String {
    format!("Meet the {}", animal.name())
}

// Trait object: a mix of types in one list, decided at run time (18.2).
fn chorus(animals: &[Box<dyn Animal>]) -> Vec<String> {
    animals.iter().map(|a| a.describe()).collect()
}

fn main() {
    println!("{}", introduce(&Dog));
    println!("{}", introduce(&Owl));

    let zoo: Vec<Box<dyn Animal>> = vec![Box::new(Dog), Box::new(Owl)];
    for line in chorus(&zoo) {
        println!("  {line}");
    }

    let car = Car {
        name: "Mini".into(),
        engine: Engine { horsepower: 90 },
    };
    println!("{}", car.start());

    // What you do NOT get, and why that is fine:
    // - no `super.describe()`: a default method can't be called once it is
    //   overridden. Put the shared part in its own method and call that.
    // - no deep class trees where a change to a parent silently changes
    //   every child. Each type opts in to each trait, one `impl` at a time.
}
