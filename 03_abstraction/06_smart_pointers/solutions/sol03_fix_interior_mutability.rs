//! Solution to exercise 3: make it compile (E0596) with `RefCell`.
//!
//! `RefCell<Vec<String>>` moves the borrow check from compile time to run
//! time. `borrow_mut()` hands out a mutable borrow through `&self`, and
//! would panic if another borrow were active at the same moment. Here each
//! borrow ends at the end of its statement, so that never happens.
//!
//! Run: cargo run -p smart_pointers --features solutions --bin sol03_fix_interior_mutability

use std::cell::RefCell;

/// Imagine this trait is from a crate you can't change.
trait Logger {
    fn log(&self, message: &str);
}

struct MemoryLogger {
    lines: RefCell<Vec<String>>,
}

impl Logger for MemoryLogger {
    fn log(&self, message: &str) {
        self.lines.borrow_mut().push(message.to_string());
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
    let logger = MemoryLogger {
        lines: RefCell::new(Vec::new()),
    };
    process_order(&logger, 12);
    let lines = logger.lines.borrow();
    println!("{} lines logged:", lines.len());
    for line in lines.iter() {
        println!("  {line}");
    }
}
