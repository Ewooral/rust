//! Book 15.2: the `Deref` trait, and deref coercion.
//!
//! Run: cargo run -p smart_pointers --example deref_my_box

use std::ops::{Deref, DerefMut};

/// The Book's `MyBox`: like `Box`, but it keeps the value on the stack.
/// It only exists to show how `*` works.
struct MyBox<T>(T);

impl<T> MyBox<T> {
    fn new(x: T) -> MyBox<T> {
        MyBox(x)
    }
}

// Without this impl, `*y` below is error[E0614]: type `MyBox<{integer}>`
// cannot be dereferenced.
impl<T> Deref for MyBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

/// `DerefMut` is the same for `&mut`: it lets `*y = ...` and `&mut self`
/// methods reach the inside.
impl<T> DerefMut for MyBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

fn hello(name: &str) {
    println!("Hello, {name}!");
}

fn main() {
    // 1. Following a plain reference with *.
    let x = 5;
    let y = &x;
    assert_eq!(5, *y);

    // 2. The same with Box and with our MyBox. `*y` really means
    //    `*(y.deref())`: deref() gives a &i32, and * follows it.
    let y = Box::new(x);
    assert_eq!(5, *y);
    let y = MyBox::new(x);
    assert_eq!(5, *y);
    assert_eq!(5, *(y.deref())); // what Rust writes for you
    println!("*MyBox::new(5) = {}", *y);

    // 3. Deref coercion: &MyBox<String> -> &String -> &str, automatically,
    //    because the function wants a &str.
    let m = MyBox::new(String::from("Rust"));
    hello(&m);
    // Without coercion we'd have to write this:
    hello(&(*m)[..]);

    // 4. DerefMut: change the String inside the box, and call &mut methods.
    let mut m = m;
    m.push_str("acean"); // String::push_str found through DerefMut
    *m = m.to_uppercase();
    hello(&m);
}

// Deref coercion rules (all decided at compile time, so they cost nothing):
//   &T     -> &U      when T: Deref<Target = U>
//   &mut T -> &mut U  when T: DerefMut<Target = U>
//   &mut T -> &U      when T: Deref<Target = U>
// Never &T -> &mut U: that would break the borrowing rules.
