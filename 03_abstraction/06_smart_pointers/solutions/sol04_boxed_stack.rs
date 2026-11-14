//! Solution to exercise 4: a stack of plates, built from boxes.
//!
//! `push` takes the old head out (`take`) and boxes a new node that owns it.
//! `pop` takes the head out, unboxes it with `*`, and makes its `next` the
//! new head. `peek` only borrows: `as_deref` looks through the `Box`.
//!
//! Test: cargo test -p smart_pointers --features solutions --bin sol04_boxed_stack

struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}

struct Stack<T> {
    head: Option<Box<Node<T>>>,
    len: usize,
}

impl<T> Stack<T> {
    fn new() -> Self {
        Stack { head: None, len: 0 }
    }

    /// Puts `value` on top.
    fn push(&mut self, value: T) {
        let below = self.head.take();
        self.head = Some(Box::new(Node { value, next: below }));
        self.len += 1;
    }

    /// Takes the top value off, or `None` if the stack is empty.
    fn pop(&mut self) -> Option<T> {
        let boxed = self.head.take()?;
        let Node { value, next } = *boxed; // move the node out of its box
        self.head = next;
        self.len -= 1;
        Some(value)
    }

    /// The top value, without taking it off.
    fn peek(&self) -> Option<&T> {
        self.head.as_deref().map(|node| &node.value)
    }

    /// How many values are on the stack.
    fn len(&self) -> usize {
        self.len
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

fn main() {
    let mut plates = Stack::new();
    for plate in ["blue", "red", "green"] {
        plates.push(plate);
    }
    println!("{} plates, top plate: {:?}", plates.len(), plates.peek());
    while let Some(plate) = plates.pop() {
        println!("took the {plate} plate");
    }
    println!("empty now? {}", plates.is_empty());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_in_first_out() {
        let mut stack = Stack::new();
        stack.push(1);
        stack.push(2);
        stack.push(3);
        assert_eq!(stack.pop(), Some(3));
        assert_eq!(stack.pop(), Some(2));
        stack.push(4);
        assert_eq!(stack.pop(), Some(4));
        assert_eq!(stack.pop(), Some(1));
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn peek_looks_without_taking() {
        let mut stack = Stack::new();
        assert_eq!(stack.peek(), None);
        stack.push(String::from("plate"));
        assert_eq!(stack.peek().map(String::as_str), Some("plate"));
        assert_eq!(stack.len(), 1, "still there");
    }

    #[test]
    fn len_follows_pushes_and_pops() {
        let mut stack = Stack::new();
        assert!(stack.is_empty());
        for i in 0..10_000 {
            stack.push(i);
        }
        assert_eq!(stack.len(), 10_000);
        while stack.pop().is_some() {}
        assert!(stack.is_empty());
    }
}
