//! Methods: functions defined in an `impl` block whose first parameter is `self`.
//! Run: cargo run -p structs --example methods

#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    /// `&self` is short for `self: &Self`: borrow to read.
    fn area(&self) -> u32 {
        self.width * self.height
    }

    /// A method may share a field's name. `rect.width()` is the method,
    /// `rect.width` is the field. Often used as a getter.
    fn width(&self) -> bool {
        self.width > 0
    }

    /// Methods can take more parameters after `self`.
    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }

    /// `&mut self`: borrow to change.
    fn double(&mut self) {
        self.width *= 2;
        self.height *= 2;
    }

    /// `self`: take ownership. Rare; used when the method turns the value
    /// into something else, so the caller should not use the old one.
    fn into_tuple(self) -> (u32, u32) {
        (self.width, self.height)
    }
}

fn main() {
    let mut rect1 = Rectangle {
        width: 30,
        height: 50,
    };
    let rect2 = Rectangle {
        width: 10,
        height: 40,
    };

    println!("area: {}", rect1.area());
    if rect1.width() {
        println!("width is non-zero: {}", rect1.width);
    }
    println!("rect1 can hold rect2? {}", rect1.can_hold(&rect2));

    // Automatic referencing: `rect1.double()` is `(&mut rect1).double()`.
    // Rust adds the &, &mut or * that the method's `self` asks for.
    rect1.double();
    println!("doubled: {rect1:?}");

    let (w, h) = rect1.into_tuple();
    println!("as a tuple: ({w}, {h})");
    // rect1.area(); // E0382: rect1 was moved into `into_tuple`
}
