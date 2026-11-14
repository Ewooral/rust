//! Book 15.1: `Box<T>` puts a value on the heap; a recursive type needs one.
//!
//! Run: cargo run -p smart_pointers --example box_cons_list

use std::mem::size_of;

// The Book's cons list. Each `Cons` holds a value and the REST of the list.
// Without the Box, a List would contain a List would contain a List...
// and have no size at all:
//
//     enum List { Cons(i32, List), Nil }   // error[E0072]: recursive type `List` has infinite size
//
// A Box is a pointer, and a pointer always has the same size, so this works:
#[derive(Debug)]
enum List {
    Cons(i32, Box<List>),
    Nil,
}

use List::{Cons, Nil};

fn sum(list: &List) -> i32 {
    match list {
        Cons(value, rest) => value + sum(rest), // &Box<List> coerces to &List
        Nil => 0,
    }
}

fn main() {
    // 1. A Box is an owner, like a variable: the 5 lives on the heap, the
    //    pointer `b` on the stack. When `b` goes out of scope, both are freed.
    let b = Box::new(5);
    println!("b = {b}"); // Box<i32> prints like the i32 inside

    // 2. The recursive list.
    let list = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))));
    println!("list = {list:?}");
    println!("sum  = {}", sum(&list));

    // 3. Sizes: a Box is one pointer wide, whatever it points to.
    println!("\nsize of i32:            {} bytes", size_of::<i32>());
    println!("size of Box<i32>:       {} bytes", size_of::<Box<i32>>());
    println!(
        "size of Box<[u8; 1000]>: {} bytes",
        size_of::<Box<[u8; 1000]>>()
    );
    println!("size of List:           {} bytes", size_of::<List>());

    // 4. Moving a Box moves only the pointer, not the data it points to:
    //    handy for large values.
    let big = Box::new([0u8; 1000]);
    let moved = big; // copies 8 bytes, not 1000
    println!("moved a {}-byte array by moving its box", moved.len());
}
