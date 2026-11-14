//! Mini project for Book 3.5: the three programs the Book suggests at the end
//! of chapter 3, plus a puzzle that needs a loop label.
//!
//! 1. Convert temperatures between Fahrenheit and Celsius.
//! 2. Generate the nth Fibonacci number.
//! 3. Print a cumulative song like "The Twelve Days of Christmas", using its
//!    repetition. Ours is "The Twelve Days of Rust", with Rust-themed gifts.
//! 4. Find a Pythagorean triple by breaking out of two loops at once.
//!
//! Run: cargo run -p control_flow
//! Test: cargo test -p control_flow

const ORDINALS: [&str; 12] = [
    "first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth", "tenth",
    "eleventh", "twelfth",
];

const GIFTS: [&str; 12] = [
    "a crab named Ferris",
    "two borrowed slices",
    "three Cargo crates",
    "four match arms",
    "five passing tests",
    "six lifetimes ticking",
    "seven traits implementing",
    "eight threads spawning",
    "nine closures capturing",
    "ten iterators chaining",
    "eleven macros expanding",
    "twelve enums matching",
];

fn main() {
    println!("== 1. Temperatures ==");
    // A range with a step: -10, 0, 10, 20, 30, 40.
    for celsius in (-10..=40).step_by(10) {
        let c = f64::from(celsius);
        println!(
            "{celsius:>4} °C = {:>5.1} °F  {}",
            celsius_to_fahrenheit(c),
            describe(c)
        );
    }
    let body = 98.6;
    println!("{body} °F = {:.1} °C", fahrenheit_to_celsius(body));

    println!("\n== 2. Fibonacci ==");
    for n in 0..=15 {
        print!("{} ", fibonacci(n));
    }
    println!(
        "\nfibonacci(93) = {} (the biggest that fits in a u64)",
        fibonacci(93)
    );

    println!("\n== 3. The Twelve Days of Rust ==");
    for day in 1..=12 {
        println!("{}", verse(day));
    }

    println!("== 4. Loop labels ==");
    for perimeter in [12, 30, 1000, 7] {
        match pythagorean_triple(perimeter) {
            Some((a, b, c)) => println!("perimeter {perimeter}: {a}² + {b}² = {c}²"),
            None => println!("perimeter {perimeter}: no whole-number right triangle"),
        }
    }
}

/// °C to °F: multiply by 9/5, then add 32.
fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    celsius * 9.0 / 5.0 + 32.0
}

/// °F to °C: subtract 32, then multiply by 5/9.
fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}

/// A word for how a temperature in °C feels. `if` is an expression, so the
/// whole `if`/`else if` chain IS the return value. The first true condition
/// wins, so the order of the checks matters.
fn describe(celsius: f64) -> &'static str {
    if celsius <= 0.0 {
        "freezing"
    } else if celsius < 10.0 {
        "cold"
    } else if celsius < 20.0 {
        "mild"
    } else if celsius < 30.0 {
        "warm"
    } else {
        "hot"
    }
}

/// The nth Fibonacci number, counting from fibonacci(0) = 0 and
/// fibonacci(1) = 1. Each number is the sum of the two before it.
///
/// Works up to n = 93. fibonacci(94) is too big for a u64, so it panics
/// with "attempt to add with overflow" in a debug build.
fn fibonacci(n: u32) -> u64 {
    if n == 0 {
        return 0;
    }
    let mut previous: u64 = 0;
    let mut current: u64 = 1;
    // Step forward n - 1 times, never computing one number further than we
    // need (that one extra number would overflow when n is 93).
    for _ in 1..n {
        let next = previous + current;
        previous = current;
        current = next;
    }
    current
}

/// One verse of the song for `day` (1 to 12): the gifts are listed from
/// today's back down to the first, which gets an "and" after day 1.
fn verse(day: usize) -> String {
    let mut text = format!(
        "On the {} day of Rust, the compiler gave to me:\n",
        ORDINALS[day - 1]
    );
    // `.rev()` counts down: day, day - 1, ..., 1.
    for gift_day in (1..=day).rev() {
        if gift_day == 1 && day > 1 {
            text.push_str("and ");
        }
        text.push_str(GIFTS[gift_day - 1]);
        text.push('\n');
    }
    text
}

/// Finds whole numbers a <= b with a² + b² = c² and a + b + c = `perimeter`.
/// Two nested loops search every (a, b); the label lets the inner loop stop
/// BOTH loops the moment a triple is found.
fn pythagorean_triple(perimeter: u32) -> Option<(u32, u32, u32)> {
    let mut found = None;
    'search: for a in 1..perimeter {
        for b in a..perimeter - a {
            let c = perimeter - a - b;
            if a * a + b * b == c * c {
                found = Some((a, b, c));
                break 'search;
            }
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn converts_known_temperatures() {
        assert!(close(celsius_to_fahrenheit(100.0), 212.0));
        assert!(close(celsius_to_fahrenheit(0.0), 32.0));
        assert!(close(fahrenheit_to_celsius(212.0), 100.0));
        assert!(close(fahrenheit_to_celsius(32.0), 0.0));
        // The one temperature that's the same on both scales:
        assert!(close(celsius_to_fahrenheit(-40.0), -40.0));
    }

    #[test]
    fn converting_there_and_back_is_the_identity() {
        for c in [-273.15, -40.0, 0.0, 21.5, 100.0] {
            assert!(close(fahrenheit_to_celsius(celsius_to_fahrenheit(c)), c));
        }
    }

    #[test]
    fn describes_temperatures_at_the_boundaries() {
        assert_eq!(describe(-5.0), "freezing");
        assert_eq!(describe(0.0), "freezing");
        assert_eq!(describe(9.9), "cold");
        assert_eq!(describe(10.0), "mild");
        assert_eq!(describe(29.9), "warm");
        assert_eq!(describe(30.0), "hot");
    }

    #[test]
    fn fibonacci_numbers() {
        let first_ten: Vec<u64> = (0..10).map(fibonacci).collect();
        assert_eq!(first_ten, [0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);
        assert_eq!(fibonacci(50), 12_586_269_025);
        assert_eq!(fibonacci(93), 12_200_160_415_121_876_738);
    }

    #[test]
    fn first_verse_has_one_gift_and_no_and() {
        assert_eq!(
            verse(1),
            "On the first day of Rust, the compiler gave to me:\na crab named Ferris\n"
        );
    }

    #[test]
    fn later_verses_count_down_and_end_with_and() {
        assert_eq!(
            verse(3),
            "On the third day of Rust, the compiler gave to me:\n\
             three Cargo crates\n\
             two borrowed slices\n\
             and a crab named Ferris\n"
        );
    }

    #[test]
    fn the_last_verse_has_every_gift() {
        let last = verse(12);
        assert_eq!(last.lines().count(), 13);
        assert!(last.starts_with("On the twelfth day"));
        assert!(last.ends_with("and a crab named Ferris\n"));
    }

    #[test]
    fn finds_pythagorean_triples() {
        assert_eq!(pythagorean_triple(12), Some((3, 4, 5)));
        assert_eq!(pythagorean_triple(30), Some((5, 12, 13)));
        assert_eq!(pythagorean_triple(1000), Some((200, 375, 425)));
        assert_eq!(pythagorean_triple(7), None);
    }
}
