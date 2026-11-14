//! Book 13.1: FnOnce, FnMut and Fn, and taking or returning closures.
//! Run: cargo run -p iterators_and_closures --example fn_traits
//!
//! Every closure implements one or more of these traits, decided by what its
//! body does with the values it captured:
//!   FnOnce  - can be called at least once. Every closure is FnOnce.
//!             A closure that MOVES a captured value out is ONLY FnOnce.
//!   FnMut   - can be called many times; may CHANGE captured values.
//!   Fn      - can be called many times; changes nothing it captured.
//! Fn is also FnMut, and FnMut is also FnOnce.

#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

// Taking closures: the bound says how the function will call it.
fn call_once<F: FnOnce() -> String>(f: F) -> String {
    f() // once, so any closure is accepted
}

fn call_twice<F: FnMut()>(mut f: F) {
    f();
    f();
}

fn apply_to_3<F: Fn(i32) -> i32>(f: F) -> i32 {
    f(3)
}

// Returning closures: each closure has its own unnamed type, so name it
// with `impl Fn`, or box it when different closures may come back.
fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n // `move`: the closure must own n, since n dies here
}

fn make_op(double: bool) -> Box<dyn Fn(i32) -> i32> {
    if double {
        Box::new(|x| x * 2)
    } else {
        Box::new(|x| x + 100)
    }
}

fn main() {
    // FnOnce only: this closure gives away the String it captured.
    let name = String::from("Ferris");
    let give_away = move || name; // moves `name` OUT when called
    println!("{}", call_once(give_away));
    // give_away(); // E0382: use of moved value: `give_away`
    //                 (an FnOnce closure is used up by its one call)

    // FnMut: changes what it captured.
    let mut count = 0;
    call_twice(|| count += 1);
    println!("count = {count}");

    // Fn: only reads.
    let offset = 10;
    println!("apply_to_3 = {}", apply_to_3(|x| x + offset));

    // The Book's example: sort_by_key calls its closure many times, so it
    // needs FnMut. A closure that pushes a captured String into a Vec moves
    // the String out on the first call, so it is FnOnce and is rejected:
    let mut list = [
        Rectangle {
            width: 10,
            height: 1,
        },
        Rectangle {
            width: 3,
            height: 5,
        },
        Rectangle {
            width: 7,
            height: 12,
        },
    ];
    // let mut sort_operations = vec![];
    // let value = String::from("closure called");
    // list.sort_by_key(|r| {
    //     sort_operations.push(value); // E0507: cannot move out of `value`, a captured variable in an `FnMut` closure
    //     r.width
    // });
    let mut num_sort_operations = 0;
    list.sort_by_key(|r| {
        num_sort_operations += 1; // counting is fine: FnMut may mutate
        r.width
    });
    println!("{list:?}, sorted in {num_sort_operations} operations");
    let areas: Vec<u32> = list.iter().map(|r| r.width * r.height).collect();
    println!("areas: {areas:?}");

    let add_five = make_adder(5);
    println!("make_adder(5)(1) = {}", add_five(1));
    let ops = [make_op(true), make_op(false)];
    for op in &ops {
        println!("op(1) = {}", op(1));
    }

    // Functions can be passed wherever a closure is expected, too.
    let words: Vec<String> = ["a", "b"].iter().map(ToString::to_string).collect();
    println!("{words:?}");
}
