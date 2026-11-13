//! Tuple structs (named type, unnamed fields) and unit-like structs (no fields).
//! Run: cargo run -p structs --example tuple_and_unit_structs

#[derive(Debug)]
struct Color(u8, u8, u8);

#[derive(Debug)]
struct Point(i32, i32, i32);

/// A unit-like struct has no data. Useful when you need a type to hang
/// behaviour (traits) on, or as a marker.
#[derive(Debug, PartialEq)]
struct AlwaysEqual;

fn print_color(c: &Color) {
    println!("rgb({}, {}, {})", c.0, c.1, c.2);
}

fn main() {
    let black = Color(0, 0, 0);
    let origin = Point(0, 0, 0);

    // Same shape, different types: a Point is not a Color.
    print_color(&black);
    // print_color(&origin); // E0308: mismatched types, expected `&Color`, found `&Point`

    // Access by index, or destructure by naming the type.
    let Point(x, y, z) = origin;
    println!("origin = ({x}, {y}, {z})");

    let subject = AlwaysEqual;
    assert_eq!(subject, AlwaysEqual);
    println!("{subject:?} takes {} bytes", size_of::<AlwaysEqual>());
}
