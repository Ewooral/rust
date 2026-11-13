//! assert!, assert_eq!, assert_ne!, and custom failure messages.
//! Run: cargo test -p automated_tests --example assertions
//! Failures: cargo test -p automated_tests --example assertions -- --ignored

#[derive(Debug, PartialEq)] // assert_eq! needs both: == to compare, Debug to print
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }
}

fn add_two(a: u64) -> u64 {
    a + 2
}

fn greeting(name: &str) -> String {
    format!("Hello {name}!")
}

fn main() {
    println!("Run me with `cargo test`: see the header.");
    let big = Rectangle {
        width: 8,
        height: 7,
    };
    let small = Rectangle {
        width: 5,
        height: 1,
    };
    println!("{big:?} can hold {small:?}? {}", big.can_hold(&small));
    println!("add_two(2) = {}, {}", add_two(2), greeting("Carol"));
}

#[cfg(test)]
mod tests {
    use super::*;

    // assert!(condition): passes when the condition is true.
    #[test]
    fn larger_can_hold_smaller() {
        let larger = Rectangle {
            width: 8,
            height: 7,
        };
        let smaller = Rectangle {
            width: 5,
            height: 1,
        };
        assert!(larger.can_hold(&smaller));
        assert!(!smaller.can_hold(&larger));
    }

    // assert_eq!(left, right) / assert_ne!(left, right): on failure, both
    // values are printed, which assert!(a == b) cannot do.
    #[test]
    fn it_adds_two() {
        assert_eq!(add_two(2), 4);
        assert_ne!(add_two(2), 5);
        assert_eq!(
            Rectangle {
                width: 1,
                height: 2
            },
            Rectangle {
                width: 1,
                height: 2
            }
        );
    }

    // Extra arguments after the condition are a format! message, shown
    // only when the assertion fails.
    #[test]
    fn greeting_contains_name() {
        let result = greeting("Carol");
        assert!(
            result.contains("Carol"),
            "Greeting did not contain name, value was `{result}`"
        );
    }

    // What a failing assert_eq! prints:
    //   assertion `left == right` failed
    //     left: 5
    //    right: 4
    #[test]
    #[ignore = "fails on purpose, to show the failure output"]
    fn failing_assert_eq() {
        assert_eq!(add_two(3), 4, "add_two(3) should be 5, so this fails");
    }
}
