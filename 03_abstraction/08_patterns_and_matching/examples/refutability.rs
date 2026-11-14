//! Book 19.2: refutability, whether a pattern might fail to match.
//! Run: cargo run -p patterns_and_matching --example refutability
//!
//! IRREFUTABLE pattern: matches every possible value.   `x`, `(a, b)`, `Point { x, y }`
//! REFUTABLE pattern:   can fail for some value.         `Some(x)`, `Ok(n)`, `0`, `[first, ..]`
//!
//! Where each kind is allowed:
//!
//!   let, function parameters, for loops     irrefutable only (no way to say "else")
//!   if let, while let, let ... else         refutable (they have a plan for "no match")
//!   match arms                              refutable, but the arms TOGETHER must cover everything

fn main() {
    let some_option_value: Option<i32> = None;

    // A refutable pattern where only irrefutable ones are allowed:
    // let Some(x) = some_option_value;
    //   // E0005: refutable pattern in local binding, pattern `None` not covered
    //   // (the compiler suggests `let...else`)

    // Fix 1: if let, when "no match" just means "skip this".
    if let Some(x) = some_option_value {
        println!("got {x}");
    } else {
        println!("if let:   nothing there, carried on");
    }

    // Fix 2: let ... else, when "no match" means "stop here".
    let value = Some(7);
    let Some(x) = value else {
        println!("no value");
        return;
    };
    println!("let-else: x = {x}");

    // The same rule for function parameters and for loops:
    // fn first(Some(x): Option<i32>) -> i32 { x }
    //   // E0005: refutable pattern in function argument
    // for Some(n) in vec![Some(1), None] {}
    //   // E0005: refutable pattern in `for` loop binding
    // Use a match inside, or filter first: `.into_iter().flatten()`.
    for n in [Some(1), None, Some(3)].into_iter().flatten() {
        println!("for:      {n}");
    }

    // The other way round: an IRREFUTABLE pattern in `if let` is legal but
    // pointless, so it's a WARNING, not an error:
    //   if let x = 5 { ... }
    //   // warning: irrefutable `if let` pattern, `#[warn(irrefutable_let_patterns)]`
    //   //          help: consider replacing the `if let` with a `let`

    // match arms: each may be refutable, but the LAST arm is often an
    // irrefutable catch-all so that together they cover every value.
    let roll = 4;
    let message = match roll {
        6 => "six!",
        1 => "one...",
        _ => "something in between", // irrefutable: matches anything left
    };
    println!("match:    {message}");
}
