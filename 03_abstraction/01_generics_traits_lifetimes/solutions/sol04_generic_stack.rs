//! Solution to exercise 4: a generic struct with methods, and a conditional method.
//!
//! Implement `Stack<T>`: last in, first out, stored in a Vec.
//! `describe` exists only for stacks whose items can be displayed: add the
//! bound to its impl block yourself.
//!
//! Test: cargo test -p generics_traits_lifetimes --features solutions --bin sol04_generic_stack

use std::fmt::Display;

#[derive(Debug, Default)]
struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    fn new() -> Self {
        Stack { items: Vec::new() }
    }

    fn push(&mut self, item: T) {
        self.items.push(item);
    }

    /// Removes and returns the top item.
    fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    /// Looks at the top item without removing it.
    fn peek(&self) -> Option<&T> {
        self.items.last()
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

// Only stacks of displayable items get `describe`.
impl<T: Display> Stack<T> {
    /// "[bottom, ..., top]", e.g. "[1, 2, 3]".
    fn describe(&self) -> String {
        let parts: Vec<String> = self.items.iter().map(|item| item.to_string()).collect();
        format!("[{}]", parts.join(", "))
    }
}

fn main() {
    let mut s = Stack::new();
    s.push(1);
    s.push(2);
    println!("{} items, top {:?}, {}", s.len(), s.peek(), s.describe());
    println!("popped {:?}, empty now? {}", s.pop(), s.is_empty());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_in_first_out() {
        let mut s = Stack::new();
        assert!(s.is_empty());
        s.push("a");
        s.push("b");
        assert_eq!(s.peek(), Some(&"b"));
        assert_eq!(s.pop(), Some("b"));
        assert_eq!(s.pop(), Some("a"));
        assert_eq!(s.pop(), None);
    }

    #[test]
    fn works_for_any_type() {
        let mut s: Stack<Vec<i32>> = Stack::new();
        s.push(vec![1]);
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn describe_lists_bottom_to_top() {
        let mut s = Stack::new();
        assert_eq!(s.describe(), "[]");
        for n in [1, 2, 3] {
            s.push(n);
        }
        assert_eq!(s.describe(), "[1, 2, 3]");
    }
}
