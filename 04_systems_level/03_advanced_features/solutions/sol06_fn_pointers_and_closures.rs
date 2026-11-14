//! Solution to exercise 6: function pointers and returning closures.
//!
//! A tiny calculator. Fill in each `todo!()`:
//!   - `lookup(symbol)` returns the matching FUNCTION POINTER from the
//!     functions below: "+" -> add, "-" -> sub, "*" -> mul, else None.
//!   - `make_scaler(factor)` returns a CLOSURE that multiplies by `factor`.
//!     (It captures `factor`, so it can't be a plain `fn` pointer.)
//!   - `pipeline(steps)` returns ONE closure that runs every step in order,
//!     feeding each result into the next. (Hint: `move` and `fold`.)
//!
//! The tests are written; make them pass.
//!
//! Test: cargo test -p advanced_features --features solutions --bin sol06_fn_pointers_and_closures

/// A type alias for a function pointer type.
type BinOp = fn(i64, i64) -> i64;

fn add(a: i64, b: i64) -> i64 {
    a + b
}

fn sub(a: i64, b: i64) -> i64 {
    a - b
}

fn mul(a: i64, b: i64) -> i64 {
    a * b
}

fn lookup(symbol: &str) -> Option<BinOp> {
    // Each function NAME coerces to the fn pointer type BinOp.
    match symbol {
        "+" => Some(add),
        "-" => Some(sub),
        "*" => Some(mul),
        _ => None,
    }
}

fn make_scaler(factor: i64) -> impl Fn(i64) -> i64 {
    move |x| x * factor // `move`: the closure keeps its own copy of factor
}

fn pipeline(steps: Vec<Box<dyn Fn(i64) -> i64>>) -> impl Fn(i64) -> i64 {
    // `move` hands the Vec to the closure, which owns it from now on.
    move |x| steps.iter().fold(x, |value, step| step(value))
}

fn main() {
    if let Some(op) = lookup("*") {
        println!("6 * 7 = {}", op(6, 7));
    }
    let triple = make_scaler(3);
    println!("triple(14) = {}", triple(14));
    let recipe = pipeline(vec![Box::new(make_scaler(2)), Box::new(|x| x + 1)]);
    println!("recipe(20) = {}", recipe(20));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_returns_function_pointers() {
        let plus = lookup("+").expect("+ is supported");
        assert_eq!(plus(2, 3), 5);
        assert_eq!(lookup("-").map(|f| f(10, 4)), Some(6));
        assert_eq!(lookup("*").map(|f| f(6, 7)), Some(42));
        assert!(lookup("%").is_none());
    }

    #[test]
    fn scaler_remembers_its_factor() {
        let double = make_scaler(2);
        let negate = make_scaler(-1);
        assert_eq!(double(21), 42);
        assert_eq!(negate(5), -5);
    }

    #[test]
    fn pipeline_runs_steps_in_order() {
        let steps: Vec<Box<dyn Fn(i64) -> i64>> = vec![
            Box::new(make_scaler(10)),
            Box::new(|x| x - 1),
            Box::new(|x| mul(x, x)), // functions can be called from closures too
        ];
        let run = pipeline(steps);
        assert_eq!(run(2), 361); // (2 * 10 - 1)^2
        let nothing = pipeline(Vec::new());
        assert_eq!(nothing(7), 7, "no steps: the value passes straight through");
    }
}
