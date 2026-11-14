//! Integration tests for our procedural macros (./macros), used exactly as
//! another crate would use them.
//!
//! Run: cargo test -p advanced_features --test proc_macros

use advanced_features::{HelloMacro, shout, trace};

#[derive(HelloMacro)]
struct Pancakes;

#[test]
fn derive_writes_the_trait_impl() {
    Pancakes::hello_macro(); // only compiles because the derive wrote the impl
}

#[test]
fn shout_upper_cases_at_compile_time() {
    const LOUD: &str = shout!("hello, world");
    assert_eq!(LOUD, "HELLO, WORLD");
    assert_eq!(
        shout!("tab\there\n"),
        "TAB\tHERE\n",
        "escapes are left alone"
    );
}

#[trace]
fn first_even(values: &[i32]) -> Option<i32> {
    for &v in values {
        if v % 2 == 0 {
            return Some(v); // an early return still passes through the guard
        }
    }
    None
}

#[trace("custom label")]
fn parse_number(text: &str) -> Result<i32, std::num::ParseIntError> {
    let n: i32 = text.trim().parse()?; // `?` still works inside a traced body
    Ok(n * 2)
}

#[test]
fn trace_keeps_the_function_behaving_the_same() {
    assert_eq!(first_even(&[1, 3, 4, 6]), Some(4));
    assert_eq!(first_even(&[]), None);
    assert_eq!(parse_number(" 21 "), Ok(42));
    assert!(parse_number("twenty").is_err());
}
