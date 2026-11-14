//! Recursive macros, evaluating arguments once, `@internal` rules and a
//! "tt muncher" that reads its input one token at a time.
//!
//! Run: cargo run -p macros_in_depth --example recursion_and_internal_rules

use std::cell::Cell;

/// Recursion: the base case goes FIRST, because rules are tried in order.
/// Each argument is bound with `let`, so it's evaluated exactly once.
macro_rules! max {
    ($x:expr) => { $x };
    ($x:expr, $($rest:expr),+) => {{
        let first = $x;
        let rest = max!($($rest),+);
        if first > rest { first } else { rest }
    }};
}

/// The same macro WITHOUT the `let`s: `$x` is pasted twice, so it runs twice.
macro_rules! careless_max {
    ($x:expr) => { $x };
    ($x:expr, $($rest:expr),+) => {
        if $x > careless_max!($($rest),+) { $x } else { careless_max!($($rest),+) }
    };
}

/// A tt muncher: a reverse-Polish calculator, `rpn!(3 4 + 2 *)` = (3 + 4) * 2.
/// Each step "munches" one token off the front and calls itself with the
/// rest. The `@stack [...]` rules are INTERNAL: the `@` marks them as
/// helpers that users aren't meant to call.
macro_rules! rpn {
    // Done: no tokens left, exactly one value on the stack.
    (@stack [$result:expr]) => { $result };
    // An operator: pop two values, push the result.
    (@stack [$b:expr, $a:expr $(, $stack:expr)*] + $($rest:tt)*) => {
        rpn!(@stack [($a + $b) $(, $stack)*] $($rest)*)
    };
    (@stack [$b:expr, $a:expr $(, $stack:expr)*] - $($rest:tt)*) => {
        rpn!(@stack [($a - $b) $(, $stack)*] $($rest)*)
    };
    (@stack [$b:expr, $a:expr $(, $stack:expr)*] * $($rest:tt)*) => {
        rpn!(@stack [($a * $b) $(, $stack)*] $($rest)*)
    };
    // A number: push it. This rule must come AFTER the operators: `-` can
    // start a negative literal, and once a `$n:literal` fragment starts
    // parsing, there is no going back. Above the `-` rule, `rpn!(10 2 3 * -)`
    // fails with "error: unexpected token: `<eof>` ... while parsing argument
    // for this `literal` macro fragment".
    (@stack [$($stack:expr),*] $n:literal $($rest:tt)*) => {
        rpn!(@stack [$n $(, $stack)*] $($rest)*)
    };
    // Anything else is a mistake: say so in words, at compile time.
    (@stack [$($stack:expr),*] $bad:tt $($rest:tt)*) => {
        compile_error!(concat!("rpn!: can't use `", stringify!($bad), "` here"))
    };
    // The public entry point: start with an empty stack.
    ($($tokens:tt)+) => { rpn!(@stack [] $($tokens)+) };
}

thread_local! {
    static CALLS: Cell<u32> = const { Cell::new(0) };
}

/// A "next number" with a side effect, so we can count evaluations.
fn next(n: i32) -> i32 {
    CALLS.with(|c| c.set(c.get() + 1));
    n
}

fn main() {
    println!("max!(3, 9, 4)          = {}", max!(3, 9, 4));

    CALLS.with(|c| c.set(0));
    let m = max!(next(1), next(5), next(2));
    println!(
        "max! with side effects = {m}, next() ran {} times",
        CALLS.with(Cell::get)
    );

    CALLS.with(|c| c.set(0));
    let m = careless_max!(next(1), next(5), next(2));
    println!(
        "careless_max!          = {m}, next() ran {} times (!)",
        CALLS.with(Cell::get)
    );

    println!("rpn!(3 4 + 2 *)        = {}", rpn!(3 4 + 2 *));
    println!("rpn!(10 2 3 * -)       = {}", rpn!(10 2 3 * -));
    // rpn!(1 2 /);
    // ^ error: rpn!: can't use `/` here       (our own compile_error!, no code)
    //   A macro can't report errors any other way: `compile_error!` is how
    //   a `macro_rules!` macro says "no" in words.

    // Each step is one level of recursion, and there's a limit:
    //     macro_rules! forever { () => { forever!() } }
    //     error: recursion limit reached while expanding `forever!`  (no error code)
    //     = help: consider increasing the recursion limit by adding a
    //             `#![recursion_limit = "256"]` attribute to your crate
}
