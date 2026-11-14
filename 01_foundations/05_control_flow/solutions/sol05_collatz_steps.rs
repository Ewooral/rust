//! Solution to exercise 5: a `while` loop that runs an unknown number of times.
//!
//! Test: cargo test -p control_flow --features solutions --bin sol05_collatz_steps

fn collatz_steps(n: u64) -> u32 {
    let mut current = n;
    let mut steps = 0;
    while current != 1 {
        current = if current.is_multiple_of(2) {
            current / 2
        } else {
            3 * current + 1
        };
        steps += 1;
    }
    steps
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
