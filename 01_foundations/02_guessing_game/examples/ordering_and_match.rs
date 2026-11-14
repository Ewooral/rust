//! Comparing two numbers with `cmp`, and choosing what to do with `match`.
//! Run: cargo run -p guessing_game --example ordering_and_match

use std::cmp::Ordering;

fn main() {
    let secret = 50;

    // `for` runs the body once for each guess in the list (lesson 05).
    for guess in [10, 50, 90] {
        // `cmp` compares two values and returns an `Ordering`: an enum with
        // exactly three variants, `Less`, `Greater` and `Equal`.
        let ordering = guess.cmp(&secret);

        // `match` checks the arms from top to bottom and runs the first one
        // whose pattern fits. Every possible value must be covered, or the
        // program does not compile (error E0004).
        let message = match ordering {
            Ordering::Less => "Too small!",
            Ordering::Greater => "Too big!",
            Ordering::Equal => "You win!",
        };
        println!("{guess} vs {secret}: {ordering:?} -> {message}");
    }

    // `_` is a catch-all pattern: "anything else". Here we only care about
    // one case.
    let guess = 3;
    match guess.cmp(&secret) {
        Ordering::Equal => println!("{guess}: spot on"),
        _ => println!("{guess}: not quite"),
    }
}
