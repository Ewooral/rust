//! Exercise 5: a `while` loop that runs an unknown number of times.
//!
//! The Collatz rule: if a number is even, halve it; if it's odd, triple it
//! and add 1. Repeat until you reach 1. Nobody has ever found a starting
//! number that doesn't reach 1, but nobody has proved it either!
//!
//! Implement `collatz_steps`: how many steps does `n` take to reach 1?
//! (1 takes 0 steps; 6 goes 6, 3, 10, 5, 16, 8, 4, 2, 1, which is 8 steps.)
//! Use `while`, and a `mut` copy of `n`.
//!
//! Test: cargo test -p control_flow --features exercises --bin ex05_collatz_steps

#![allow(unused_variables)]

fn collatz_steps(n: u64) -> u32 {
    todo!("count the steps from {n} down to 1")
}

fn main() {
    for n in [1, 6, 7, 27] {
        println!("{n} reaches 1 in {} steps", collatz_steps(n));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_takes_no_steps() {
        assert_eq!(collatz_steps(1), 0);
    }

    #[test]
    fn small_numbers() {
        assert_eq!(collatz_steps(2), 1);
        assert_eq!(collatz_steps(6), 8);
        assert_eq!(collatz_steps(7), 16);
    }

    #[test]
    fn twenty_seven_takes_a_long_walk() {
        assert_eq!(collatz_steps(27), 111);
    }
}
