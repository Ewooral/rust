//! Stage 3b: functions, closures and recursion, run through the library.
//! Run: cargo run -p capstone_interpreter --example stage5_closures_and_recursion

use capstone_interpreter::Interpreter;

const PROGRAM: &str = r#"
// Recursion: fib calls itself. It can, because by the time it is CALLED,
// the name `fib` is defined in the scope the function was created in.
fn fib(n) {
    if n < 2 { return n; }
    return fib(n - 1) + fib(n - 2);
}
print("fib(15) =", fib(15));

// A closure: the inner function keeps `count` alive after make_counter returns.
fn make_counter() {
    let count = 0;
    return fn () {
        count = count + 1;
        return count;
    };
}
let tick = make_counter();
let tock = make_counter();     // a separate count
tick(); tick();
print("tick:", tick(), "tock:", tock());

// Functions are values: pass them around like numbers.
fn twice(f, x) { return f(f(x)); }
print("twice(add3, 10) =", twice(fn (n) { return n + 3; }, 10));

// Lexical scope: a function sees the variables where it was WRITTEN,
// not where it is called.
let who = "global";
fn show_who() { return who; }
fn caller() {
    let who = "caller's own";
    return show_who();
}
print("show_who() from caller():", caller());
"#;

fn main() {
    let mut kite = Interpreter::new(std::io::stdout());
    if let Err(e) = kite.run(PROGRAM) {
        eprintln!("{}", e.render(PROGRAM));
    }
}
