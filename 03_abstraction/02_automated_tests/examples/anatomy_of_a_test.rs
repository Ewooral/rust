//! The anatomy of a test, and what a FAILING test looks like.
//!
//! Examples are programs, but they can contain tests too. Run them with:
//!   cargo test -p automated_tests --example anatomy_of_a_test
//! See the deliberately failing test as well:
//!   cargo test -p automated_tests --example anatomy_of_a_test -- --ignored

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

fn main() {
    println!("This file is about its tests: run it with `cargo test`, see the header.");
    println!("add(2, 2) = {}", add(2, 2));
}

// `#[cfg(test)]`: compile this module only when running tests, so it never
// ends up in the real program.
#[cfg(test)]
mod tests {
    use super::*; // bring the code under test into scope

    // `#[test]` marks a test function. It passes if it returns without panicking.
    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    // A test fails when anything in it panics: a failed assert, an unwrap on
    // None, or an explicit panic!. Each test runs in its own thread, so one
    // failure does not stop the others.
    #[test]
    #[ignore = "fails on purpose: run with -- --ignored to see a failure report"]
    fn another() {
        panic!("Make this test fail");
    }
}
