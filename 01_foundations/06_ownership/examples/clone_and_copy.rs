//! `Clone` is an explicit, possibly expensive deep copy.
//! `Copy` is an implicit, cheap bitwise copy for stack-only types.
//! Run: cargo run -p ownership --example clone_and_copy

#[derive(Debug, Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone)] // cannot be Copy: it owns a String
struct User {
    name: String,
    age: u8,
}

fn main() {
    let s1 = String::from("hello");
    let s2 = s1.clone(); // heap data duplicated
    println!("s1 = {s1}, s2 = {s2}");

    let p1 = Point { x: 1, y: 2 };
    let p2 = p1; // Copy: p1 stays valid
    println!("p1 = ({}, {}), p2 = {p2:?}", p1.x, p1.y);

    let u1 = User {
        name: "Ada".into(),
        age: 36,
    };
    let u2 = u1.clone();
    println!("u1 = {} ({}), u2 = {u2:?}", u1.name, u1.age);

    // Copy types: integers, floats, bool, char, shared references (&T),
    // and tuples/arrays made only of Copy types, e.g. (i32, bool) or [u8; 4].
    let t = (1, true);
    let t2 = t;
    println!("{t:?} {t2:?}");
}
