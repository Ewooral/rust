//! Exercise 3: make it compile (E0596) with `RefCell`.
//!
//! Pretend the `Logger` trait comes from a crate you can't change: `log`
//! takes `&self`. `MemoryLogger` wants to remember every message, but
//! pushing onto a `Vec` needs `&mut`.
//!
//! Don't change the trait. Wrap the `Vec` in a `RefCell`: push with
//! `self.lines.borrow_mut().push(...)` and read with `.borrow()`. (This is
//! the Book's MockMessenger.)
//!
//! Run: cargo run -p smart_pointers --features exercises --bin ex03_fix_interior_mutability

/// Imagine this trait is from a crate you can't change.
trait Logger {
    fn log(&self, message: &str);
}

struct MemoryLogger {
    lines: Vec<String>,
}

impl Logger for MemoryLogger {
    fn log(&self, message: &str) {
        self.lines.push(message.to_string());
    }
}

/// Code that only knows about the trait.
fn process_order(logger: &impl Logger, items: u32) {
    logger.log(&format!("order received: {items} items"));
    if items > 10 {
        logger.log("big order: call the warehouse");
    }
    logger.log("order done");
}

fn main() {
    let logger = MemoryLogger { lines: Vec::new() };
    process_order(&logger, 12);
    println!("{} lines logged:", logger.lines.len());
    for line in &logger.lines {
        println!("  {line}");
    }
}
