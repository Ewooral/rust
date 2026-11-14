//! Beyond the Book: slice and array patterns.
//! Run: cargo run -p patterns_and_matching --example slice_patterns
//!
//! `[a, b]`            exactly two elements
//! `[first, ..]`       at least one; bind the first
//! `[.., last]`        at least one; bind the last
//! `[first, .., last]` at least two; bind both ends
//! `[head, tail @ ..]` the first element, and the REST as a slice
//! `[]`                empty
//!
//! They work on arrays (fixed length, known at compile time) and on slices
//! `&[T]` (any length). For a Vec, match on `v.as_slice()`.

fn describe(numbers: &[i32]) -> String {
    match numbers {
        [] => "empty".to_string(),
        [one] => format!("just {one}"),
        [first, second] => format!("a pair: {first} and {second}"),
        [first, .., last] => format!("{} numbers, from {first} to {last}", numbers.len()),
    }
    // No `_` needed: [], [x], [x, y] and [x, .., y] cover every length.
    // Leave out the last arm and the compiler names what's missing:
    //   // E0004: non-exhaustive patterns: `&[_, _, _, ..]` not covered
}

/// Recursion on `[head, tail @ ..]`: "the first, then the rest".
fn sum(numbers: &[i32]) -> i32 {
    match numbers {
        [] => 0,
        [head, tail @ ..] => head + sum(tail),
    }
}

/// Both ends at once, then the middle: a palindrome check.
fn is_palindrome(chars: &[char]) -> bool {
    match chars {
        [] | [_] => true,
        [first, middle @ .., last] => first == last && is_palindrome(middle),
    }
}

/// Matching a command line split into words: the classic real-world use.
fn run(words: &[&str]) -> String {
    match words {
        ["help"] => "commands: add <a> <b>, greet <names...>".to_string(),
        ["add", a, b] => match (a.parse::<i32>(), b.parse::<i32>()) {
            (Ok(a), Ok(b)) => format!("{a} + {b} = {}", a + b),
            _ => format!("can't add {a:?} and {b:?}"),
        },
        ["greet", names @ ..] if !names.is_empty() => format!("hello, {}!", names.join(" and ")),
        ["greet"] => "greet whom?".to_string(),
        [unknown, ..] => format!("unknown command {unknown:?}"),
        [] => "nothing to do".to_string(),
    }
}

fn main() {
    for numbers in [&[][..], &[7], &[1, 2], &[3, 1, 4, 1, 5]] {
        println!("describe:  {numbers:?} -> {}", describe(numbers));
    }

    println!("sum:       {}", sum(&[1, 2, 3, 4]));

    for word in ["level", "rust", "a"] {
        let chars: Vec<char> = word.chars().collect();
        println!("palindrome: {word:?} -> {}", is_palindrome(&chars));
    }

    // Arrays have a fixed length, so this `let` is IRREFUTABLE: it always matches.
    let [red, green, blue] = [255, 128, 0];
    println!("array:     r={red} g={green} b={blue}");
    let [first, rest @ ..] = [1, 2, 3, 4]; // `rest` is an array [i32; 3]
    println!("array:     first {first}, rest {rest:?}");

    // On a slice, the length is unknown, so the same `let` would be refutable:
    // let [a, b] = numbers_slice;
    //   // E0005: refutable pattern in local binding
    let v = vec![10, 20, 30];
    if let [a, b, ..] = v.as_slice() {
        println!("vec:       starts with {a}, {b}");
    }

    for line in [
        "help",
        "add 2 40",
        "add two 40",
        "greet Ama Kofi",
        "greet",
        "fly",
        "",
    ] {
        let words: Vec<&str> = line.split_whitespace().collect();
        println!("command:   {line:?} -> {}", run(&words));
    }
}
