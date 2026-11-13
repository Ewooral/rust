//! Controlling how tests run: output, filtering, ignoring, threads.
//!
//! Arguments BEFORE `--` go to cargo, arguments AFTER `--` go to the test
//! binary. Try each of these:
//!
//!   cargo test -p automated_tests --example controlling_tests
//!   cargo test -p automated_tests --example controlling_tests -- --show-output
//!   cargo test -p automated_tests --example controlling_tests -- add          (names containing "add")
//!   cargo test -p automated_tests --example controlling_tests -- one_hundred  (one test)
//!   cargo test -p automated_tests --example controlling_tests -- --ignored    (only ignored ones)
//!   cargo test -p automated_tests --example controlling_tests -- --include-ignored
//!   cargo test -p automated_tests --example controlling_tests -- --test-threads=1
//!   cargo test -p automated_tests --example controlling_tests -- --list       (just list them)

fn prints_and_returns_10(a: i32) -> i32 {
    println!("I got the value {a}");
    10
}

fn add_two(a: u64) -> u64 {
    a + 2
}

fn main() {
    println!("Run me with `cargo test`: see the header.");
    println!("{} {}", prints_and_returns_10(1), add_two(2));
}

#[cfg(test)]
mod tests {
    use super::*;

    // Output from PASSING tests is captured (hidden). Add --show-output to
    // see it. Output from FAILING tests is always shown.
    #[test]
    fn this_test_prints() {
        let value = prints_and_returns_10(4);
        assert_eq!(value, 10);
    }

    // Filtering matches any part of the full test name, e.g. "add" runs
    // the three tests below. Module names count too: "tests::" runs all.
    #[test]
    fn add_two_and_two() {
        assert_eq!(add_two(2), 4);
    }

    #[test]
    fn add_three_and_two() {
        assert_eq!(add_two(3), 5);
    }

    #[test]
    fn one_hundred() {
        assert_eq!(add_two(100), 102);
    }

    // Slow tests: skip them normally, run them on purpose.
    #[test]
    #[ignore = "slow: about a second"]
    fn expensive_test() {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        assert_eq!(add_two(1), 3);
    }

    // Tests run IN PARALLEL by default. Tests that share something (a file,
    // an environment variable, a port) can collide: give each its own
    // resource, as here, or run with --test-threads=1.
    #[test]
    fn writes_its_own_file() {
        let path = std::env::temp_dir().join("controlling_tests_own_file.txt");
        std::fs::write(&path, "only this test uses this file").expect("write");
        let back = std::fs::read_to_string(&path).expect("read");
        std::fs::remove_file(&path).expect("clean up");
        assert!(back.starts_with("only this test"));
    }
}
