//! Exercise 6: the three kinds of `self`.
//!
//! Each method's comment says what it must do. Pick `&self`, `&mut self` or
//! `self` for each, and implement it. `Counter::new` is done for you.
//!
//! Test: cargo test -p structs --features exercises --bin ex06_counter

#![allow(unused_variables)]

#[derive(Debug)]
struct Counter {
    label: String,
    count: u32,
}

impl Counter {
    fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            count: 0,
        }
    }

    /// Reads the count.
    fn get(/* ? */) -> u32 {
        todo!()
    }

    /// Adds one.
    fn increment(/* ? */) {
        todo!()
    }

    /// Back to zero.
    fn reset(/* ? */) {
        todo!()
    }

    /// Finishes counting: returns "label: count" and the counter is gone.
    fn finish(/* ? */) -> String {
        todo!()
    }
}

fn main() {
    let mut laps = Counter::new("laps");
    laps.increment();
    laps.increment();
    println!("so far: {}", laps.get());
    laps.reset();
    laps.increment();
    println!("{}", laps.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_and_resets() {
        let mut c = Counter::new("x");
        assert_eq!(c.get(), 0);
        c.increment();
        c.increment();
        c.increment();
        assert_eq!(c.get(), 3);
        c.reset();
        assert_eq!(c.get(), 0);
    }

    #[test]
    fn finish_reports_label_and_count() {
        let mut c = Counter::new("goals");
        c.increment();
        assert_eq!(c.finish(), "goals: 1");
    }
}
