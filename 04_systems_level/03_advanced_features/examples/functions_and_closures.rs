//! Book 20.4: advanced functions and closures.
//! Run: cargo run -p advanced_features --example functions_and_closures
//!
//! 1. function pointers: the type `fn`   2. returning closures

// ---- 1. Function pointers ------------------------------------------------------
fn add_one(x: i32) -> i32 {
    x + 1
}

// `fn(i32) -> i32` is a TYPE: a pointer to a function. Lowercase `fn`, not
// the `Fn` trait. It is one plain address, it captures nothing.
fn do_twice(f: fn(i32) -> i32, arg: i32) -> i32 {
    f(arg) + f(arg)
}

// `fn` pointers implement all three closure traits (Fn, FnMut, FnOnce), so a
// function accepting `impl Fn` takes functions AND closures. Prefer this in
// your own APIs; take `fn` only when you must (e.g. passing it to C).
fn do_twice_generic(f: impl Fn(i32) -> i32, arg: i32) -> i32 {
    f(arg) + f(arg)
}

// A type alias keeps fn pointer types readable.
type Op = fn(i32, i32) -> i32;

#[allow(dead_code)] // we only print these
#[derive(Debug)]
enum Status {
    Value(u32),
    Stop,
}

// ---- 2. Returning closures -----------------------------------------------------
// Each closure has its own unnamed type, so the return type is "some type
// that implements Fn": impl Fn.
fn returns_closure() -> impl Fn(i32) -> i32 {
    |x| x + 1
}

fn returns_initialized_closure(init: i32) -> impl Fn(i32) -> i32 {
    move |x| x + init
}

// Two different closures can't share one `impl Fn` type:
//     fn make(add: bool, n: i32) -> impl Fn(i32) -> i32 {
//         if add { move |x| x + n } else { move |x| x * n }
//     }   // E0308: `if` and `else` have incompatible types
// Boxing them as trait objects gives them one type, chosen at run time:
fn make(add: bool, n: i32) -> Box<dyn Fn(i32) -> i32> {
    if add {
        Box::new(move |x| x + n)
    } else {
        Box::new(move |x| x * n)
    }
}

fn main() {
    println!("== 1. function pointers ==");
    println!("do_twice(add_one, 5) = {}", do_twice(add_one, 5));
    // A closure that captures NOTHING can become a fn pointer too:
    println!("do_twice(|x| x * 10, 5) = {}", do_twice(|x| x * 10, 5));
    let offset = 3;
    // do_twice(|x| x + offset, 5); // E0308: expected fn pointer, found closure (it captures `offset`)
    println!(
        "do_twice_generic(|x| x + offset, 5) = {}",
        do_twice_generic(|x| x + offset, 5)
    );

    // A function's name can stand in for a closure:
    let numbers = [1, 2, 3];
    let as_closure: Vec<String> = numbers.iter().map(|i| i.to_string()).collect();
    let as_function: Vec<String> = numbers.iter().map(ToString::to_string).collect();
    println!("{as_closure:?} == {as_function:?}");

    // A tuple struct or enum variant's NAME is a constructor function, too.
    let statuses: Vec<Status> = (0u32..3).map(Status::Value).collect();
    println!("{statuses:?}");

    // A table of fn pointers: pick behaviour by name at run time.
    // Non-capturing closures and functions (even generic std::cmp::max) mix.
    let operations: [(&str, Op); 3] = [
        ("add", |a, b| a + b),
        ("max", std::cmp::max),
        ("pow", |a, b| a.pow(b as u32)),
    ];
    for (name, op) in operations {
        println!("{name}(2, 5) = {}", op(2, 5));
    }

    println!("\n== 2. returning closures ==");
    let handlers: Vec<Box<dyn Fn(i32) -> i32>> = vec![
        Box::new(returns_closure()),
        Box::new(returns_initialized_closure(123)),
        make(false, 4),
    ];
    // vec![returns_closure(), returns_initialized_closure(123)] without boxes is E0308:
    // "distinct uses of `impl Trait` result in different opaque types"
    for handler in &handlers {
        println!("handler(5) = {}", handler(5));
    }
}
