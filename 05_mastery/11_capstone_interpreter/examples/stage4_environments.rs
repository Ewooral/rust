//! Stage 3a: environments. Each scope is an `Rc<RefCell<Env>>`: shared by
//! everyone who needs it (Rc), and changeable through any of those handles
//! (RefCell). This is lesson 03/06's `Rc<RefCell<T>>` doing real work.
//! Run: cargo run -p capstone_interpreter --example stage4_environments

use std::rc::Rc;

use capstone_interpreter::{Env, Value};

fn show(label: &str, value: Option<Value>) {
    match value {
        Some(v) => println!("{label:<34} {v:?}"),
        None => println!("{label:<34} (not defined)"),
    }
}

fn main() {
    // let x = 1;
    let global = Env::new_global();
    global.borrow_mut().define("x", Value::Number(1.0));

    // { let x = 2; let y = 3; ... }  a block: a child scope
    let block = Env::child(&global);
    block.borrow_mut().define("x", Value::Number(2.0)); // shadows, doesn't change
    block.borrow_mut().define("y", Value::Number(3.0));
    show("inside the block, x =", block.borrow().get("x"));
    show("inside the block, y =", block.borrow().get("y"));
    show("outside, x =", global.borrow().get("x"));
    show("outside, y =", global.borrow().get("y"));

    // x = 10;  inside ANOTHER block: assignment walks out to the global x
    let other = Env::child(&global);
    other.borrow_mut().assign("x", Value::Number(10.0));
    show("after `x = 10` in another block:", global.borrow().get("x"));

    // Who owns the global scope? `global`, plus every child's parent link.
    println!(
        "\nRc::strong_count(&global) = {} (us + 2 child scopes)",
        Rc::strong_count(&global)
    );
    drop(block);
    drop(other);
    println!(
        "after the blocks end:        {} (only us)",
        Rc::strong_count(&global)
    );

    // A closure keeps its scope alive after the code that made it is done:
    // here the child scope outlives the variable `outer`.
    let kept = {
        let outer = Env::new_global();
        outer
            .borrow_mut()
            .define("secret", Value::Str("still here".into()));
        Env::child(&outer)
    };
    show("\nthe outer scope lives on:", kept.borrow().get("secret"));

    // RefCell checks borrows at RUN time. Two mutable borrows at once panic.
    // (The empty panic hook just keeps the panic message off the screen.)
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(|| {
        let scope = Env::new_global();
        let _first = scope.borrow_mut();
        let _second = scope.borrow_mut(); // panics: already mutably borrowed
    });
    let _ = std::panic::take_hook();
    println!("two borrow_mut() at once -> panicked: {}", result.is_err());
    println!("(the interpreter evaluates first, THEN borrows, to avoid this)");
}
