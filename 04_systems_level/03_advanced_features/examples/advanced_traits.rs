//! Book 20.2: advanced traits.
//! Run: cargo run -p advanced_features --example advanced_traits
//!
//! 1. associated types        2. default type parameters and operator overloading
//! 3. fully qualified syntax  4. supertraits   5. the newtype pattern for foreign traits

use std::fmt;
use std::ops::Add;

// ---- 1. Associated types ------------------------------------------------------
// One implementation per type, and the type picks its Item ONCE.
struct Counter {
    count: u32,
}

impl Iterator for Counter {
    type Item = u32; // the associated type
    fn next(&mut self) -> Option<u32> {
        if self.count < 5 {
            self.count += 1;
            Some(self.count)
        } else {
            None
        }
    }
}

// Compare a GENERIC trait: a type could implement it many times, once per T,
// and every call would have to say which one it means.
trait Converter<T> {
    fn convert(&self) -> T;
}
impl Converter<String> for u32 {
    fn convert(&self) -> String {
        format!("#{self}")
    }
}
impl Converter<f64> for u32 {
    fn convert(&self) -> f64 {
        f64::from(*self) / 2.0
    }
}

// ---- 2. Default type parameters and operator overloading ----------------------
// std declares `trait Add<Rhs = Self> { type Output; fn add(self, rhs: Rhs) -> Output; }`
#[derive(Debug, Copy, Clone, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

impl Add for Point {
    // Rhs defaults to Self: Point + Point
    type Output = Point;
    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

#[derive(Debug)]
struct Millimeters(u32);
#[derive(Debug)]
struct Meters(u32);

impl Add<Meters> for Millimeters {
    // Rhs chosen explicitly: Millimeters + Meters
    type Output = Millimeters;
    fn add(self, other: Meters) -> Millimeters {
        Millimeters(self.0 + other.0 * 1000)
    }
}

// ---- 3. Fully qualified syntax ---------------------------------------------------
trait Pilot {
    fn fly(&self);
}
trait Wizard {
    fn fly(&self);
}
struct Human;

impl Pilot for Human {
    fn fly(&self) {
        println!("This is your captain speaking.");
    }
}
impl Wizard for Human {
    fn fly(&self) {
        println!("Up!");
    }
}
impl Human {
    fn fly(&self) {
        println!("*waving arms furiously*");
    }
}

trait Animal {
    fn baby_name() -> String; // no `self`: an associated FUNCTION
}
struct Dog;
impl Dog {
    fn baby_name() -> String {
        String::from("Spot")
    }
}
impl Animal for Dog {
    fn baby_name() -> String {
        String::from("puppy")
    }
}

// ---- 4. Supertraits ------------------------------------------------------------------
// "To be OutlinePrint you must already be Display", so the body may use `{self}`.
trait OutlinePrint: fmt::Display {
    fn outline_print(&self) {
        let output = self.to_string();
        let len = output.len();
        println!("{}", "*".repeat(len + 4));
        println!("*{}*", " ".repeat(len + 2));
        println!("* {output} *");
        println!("*{}*", " ".repeat(len + 2));
        println!("{}", "*".repeat(len + 4));
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
// Without the Display impl above:
// E0277: `Point` doesn't implement `std::fmt::Display`
impl OutlinePrint for Point {}

// ---- 5. Newtype pattern for foreign traits on foreign types -----------------------
// impl fmt::Display for Vec<String> {}  // E0117: only traits defined in the current crate…
struct Wrapper(Vec<String>);

impl fmt::Display for Wrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.0.join(", "))
    }
}

fn main() {
    println!("== 1. associated types ==");
    let total: u32 = Counter { count: 0 }.sum();
    println!("Counter sums to {total}");
    // let x = 7u32.convert(); // E0283: type annotations needed (which Converter<T>?)
    let label: String = 7u32.convert();
    let half: f64 = 7u32.convert();
    println!("generic trait, two impls: {label} and {half}");

    println!("\n== 2. operator overloading ==");
    let p = Point { x: 1, y: 0 } + Point { x: 2, y: 3 };
    println!("Point + Point = {p:?}");
    println!("Millimeters + Meters = {:?}", Millimeters(500) + Meters(2));
    // Meters(2) + Millimeters(500); // E0369: cannot add `Millimeters` to `Meters`

    println!("\n== 3. fully qualified syntax ==");
    let person = Human;
    person.fly(); // the inherent method wins
    Pilot::fly(&person); // pick a trait's method: the &self tells Rust which type
    Wizard::fly(&person);
    // With no inherent `fly`, `person.fly()` would be E0034: multiple applicable items in scope.

    println!("A baby dog is called a {}", Dog::baby_name());
    // Which type's baby_name? An associated function has no `self` to say:
    // println!("{}", Animal::baby_name()); // E0790
    println!("A baby dog is called a {}", <Dog as Animal>::baby_name());

    println!("\n== 4. supertraits ==");
    Point { x: 1, y: 3 }.outline_print();

    println!("\n== 5. newtype ==");
    let w = Wrapper(vec![String::from("hello"), String::from("world")]);
    println!("w = {w}");
}
