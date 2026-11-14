//! The game loop, without the keyboard: `loop`, `continue` and `break`.
//! Run: cargo run -p guessing_game --example loop_break_continue
//!
//! The "player" types these lines in order, so the output is the same every
//! time you run it.

use std::cmp::Ordering;

fn main() {
    let secret_number: u32 = 37;
    let typed = ["fifty", "50", "", "25", "37", "never reached"];
    let mut next = 0;

    // `loop` repeats forever, until something `break`s out of it.
    loop {
        let input = typed[next];
        next += 1;
        println!("player typed {input:?}");

        let guess: u32 = match input.trim().parse() {
            Ok(num) => num,
            // `continue` skips the rest of this pass and starts the next one.
            Err(_) => {
                println!("  not a number, try again");
                continue;
            }
        };

        match guess.cmp(&secret_number) {
            Ordering::Less => println!("  Too small!"),
            Ordering::Greater => println!("  Too big!"),
            Ordering::Equal => {
                println!("  You win!");
                // `break` leaves the loop for good.
                break;
            }
        }
    }

    println!("left the loop after reading {next} lines");
}
