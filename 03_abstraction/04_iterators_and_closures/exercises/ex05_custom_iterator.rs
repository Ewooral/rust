//! Exercise 5 (Beyond the Book): implement Iterator for your own type.
//!
//! `Countdown::new(3)` should yield 3, 2, 1 and then stop (return None).
//! Implement `next`. Then notice that the tests use `collect`, `sum`,
//! `filter` and `zip` without you writing any of them: they come for free
//! with Iterator.
//!
//! Test: cargo test -p iterators_and_closures --features exercises --bin ex05_custom_iterator

#![allow(unused_variables, dead_code)]

struct Countdown {
    remaining: u32,
}

impl Countdown {
    fn new(from: u32) -> Self {
        Countdown { remaining: from }
    }
}

impl Iterator for Countdown {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        todo!()
    }
}

fn main() {
    for n in Countdown::new(5) {
        println!("{n}...");
    }
    println!("lift-off!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_down_then_stops() {
        let mut c = Countdown::new(2);
        assert_eq!(c.next(), Some(2));
        assert_eq!(c.next(), Some(1));
        assert_eq!(c.next(), None);
        assert_eq!(c.next(), None, "stays finished");
    }

    #[test]
    fn adapters_come_for_free() {
        assert_eq!(Countdown::new(4).collect::<Vec<_>>(), [4, 3, 2, 1]);
        assert_eq!(Countdown::new(4).sum::<u32>(), 10);
        assert_eq!(Countdown::new(6).filter(|n| n.is_multiple_of(2)).count(), 3);
        let pairs: Vec<(u32, char)> = Countdown::new(3).zip("abc".chars()).collect();
        assert_eq!(pairs, [(3, 'a'), (2, 'b'), (1, 'c')]);
    }

    #[test]
    fn zero_yields_nothing() {
        assert_eq!(Countdown::new(0).next(), None);
    }
}
