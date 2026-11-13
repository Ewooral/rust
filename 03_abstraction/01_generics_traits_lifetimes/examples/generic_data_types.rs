//! Book 10.1: generic functions, structs, enums and methods.
//! Run: cargo run -p generics_traits_lifetimes --example generic_data_types

// ---- Generic function ----
// `<T: PartialOrd>`: for any type T whose values can be compared with `>`.
// Without the bound: E0369 binary operation `>` cannot be applied to type `&T`.
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

// ---- Generic structs ----
#[derive(Debug)]
struct Point<T> {
    x: T,
    y: T, // same T: x and y must be the same type
}

#[derive(Debug)]
struct MixedPoint<T, U> {
    x: T,
    y: U, // two parameters: x and y may differ
}

// ---- Generic methods ----
// `impl<T>` declares T, so `Point<T>` means "Point of any T".
impl<T> Point<T> {
    fn x(&self) -> &T {
        &self.x
    }
}

// A method only for ONE concrete type: Point<f32> has it, Point<i32> does not.
impl Point<f32> {
    fn distance_from_origin(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}

// Method generics can differ from the struct's generics.
impl<X1, Y1> MixedPoint<X1, Y1> {
    fn mixup<X2, Y2>(self, other: MixedPoint<X2, Y2>) -> MixedPoint<X1, Y2> {
        MixedPoint {
            x: self.x,
            y: other.y,
        }
    }
}

// ---- Generic enums: you have used two already ----
//   enum Option<T>    { Some(T), None }
//   enum Result<T, E> { Ok(T), Err(E) }

fn main() {
    println!("largest i32: {}", largest(&[34, 50, 25, 100, 65]));
    println!("largest char: {}", largest(&['y', 'm', 'a', 'q']));
    println!("largest &str: {}", largest(&["kiwi", "apple", "pear"]));

    let integer = Point { x: 5, y: 10 };
    let float = Point { x: 1.0f32, y: 4.0 };
    // let wont_work = Point { x: 5, y: 4.0 }; // E0308: expected integer, found floating-point number
    println!("{integer:?} x = {}", integer.x());
    println!("{float:?} distance = {}", float.distance_from_origin());
    // integer.distance_from_origin(); // E0599: method not found in `Point<{integer}>`

    let both = MixedPoint { x: 5, y: 4.0 };
    let p2 = MixedPoint { x: "Hello", y: 'c' };
    println!("mixup: {:?}", both.mixup(p2));

    // Monomorphization: at compile time Rust writes a separate copy of each
    // generic function for every concrete type used (largest::<i32>,
    // largest::<char>, largest::<&str>). Generic code costs NOTHING at run
    // time compared with writing those copies by hand.
    let some_number: Option<i32> = Some(5);
    let some_float: Option<f64> = Some(5.0);
    println!("{some_number:?} {some_float:?}");
}
