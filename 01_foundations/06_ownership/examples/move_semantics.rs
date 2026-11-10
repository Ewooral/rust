//! Assigning a heap-owning value moves ownership; the old name becomes unusable.
//! Run: cargo run -p ownership --example move_semantics

fn main() {
    // Integers live entirely on the stack and are `Copy`: assignment copies.
    let x = 5;
    let y = x;
    println!("x = {x}, y = {y}"); // both still usable

    // A `String` owns a heap buffer. Assignment moves the (ptr, len, capacity)
    // triple on the stack; it does NOT copy the heap data.
    let s1 = String::from("hello");
    let s2 = s1;
    println!("s2 = {s2}");

    // Uncomment the next line to see error E0382 "borrow of moved value: `s1`".
    // println!("s1 = {s1}");

    // Assigning a new value drops the old one immediately.
    let mut s = String::from("first");
    println!("s = {s}");
    s = String::from("second"); // "first" is freed here
    println!("s = {s}");
}
