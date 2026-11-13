//! The Book's section 5.2: refactoring towards a struct, then printing it.
//! Run: cargo run -p structs --example rectangles

// Version 1: two loose numbers. Nothing says they belong together.
fn area_v1(width: u32, height: u32) -> u32 {
    width * height
}

// Version 2: a tuple groups them, but which is width? `.0` says nothing.
fn area_v2(dimensions: (u32, u32)) -> u32 {
    dimensions.0 * dimensions.1
}

// Version 3: a struct. Grouped AND named.
#[derive(Debug)] // without this, `{:?}` fails with E0277
struct Rectangle {
    width: u32,
    height: u32,
}

// Borrow, so the caller keeps the rectangle.
fn area(rect: &Rectangle) -> u32 {
    rect.width * rect.height
}

fn main() {
    println!("v1: {}", area_v1(30, 50));
    println!("v2: {}", area_v2((30, 50)));

    let scale = 2;
    let rect = Rectangle {
        // dbg! prints file, line and value to stderr, then RETURNS the value,
        // so it can wrap any expression.
        width: dbg!(30 * scale),
        height: 50,
    };
    println!("v3: {}", area(&rect));

    println!("{rect:?}"); // one line
    println!("{rect:#?}"); // pretty, one field per line
    dbg!(&rect); // pass a reference so dbg! does not take ownership
}
