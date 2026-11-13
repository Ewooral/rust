//! Associated functions: functions in an `impl` block without `self`.
//! Run: cargo run -p structs --example associated_functions

#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    /// Associated constants live in the impl too.
    const UNIT: Rectangle = Rectangle {
        width: 1,
        height: 1,
    };

    /// Convention: a constructor called `new`. `Self` means `Rectangle`.
    fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Named constructors say what kind of value you get.
    fn square(size: u32) -> Self {
        Self {
            width: size,
            height: size,
        }
    }
}

// A type may have several impl blocks. Here it only splits the code up;
// it matters more with generics and traits (lesson 03/01).
impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }
}

fn main() {
    // Called with `::`, because there is no instance yet.
    let r = Rectangle::new(3, 4);
    let sq = Rectangle::square(5);
    println!("{r:?} area {}", r.area());
    println!("{sq:?} area {}", sq.area());
    println!("unit: {:?}", Rectangle::UNIT);

    // A method is an associated function too; these two lines are the same.
    println!("{}", sq.area());
    println!("{}", Rectangle::area(&sq));

    // You have used associated functions since lesson 01: String::from, Vec::new.
    let s = String::from("hello");
    println!("{s}");
}
