use colored::Colorize;
use rand::Rng;
use std::cmp::Ordering;
use std::io;

const MIN: u32 = 1;
const MAX: u32 = 100;

fn main() {
    println!("Guess the number between {MIN} and {MAX}!");
    let secret_number = rand::rng().random_range(MIN..=MAX);
    let mut attempts = 0;

    loop {
        println!("Please input your guess.");

        let mut guess = String::new();
        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");

        let guess: u32 = match guess.trim().parse() {
            Ok(num) if (MIN..=MAX).contains(&num) => num,
            _ => {
                println!(
                    "{}",
                    format!("Enter a whole number from {MIN} to {MAX}.").yellow()
                );
                continue;
            }
        };
        attempts += 1;
        println!("You guessed: {guess}");

        match guess.cmp(&secret_number) {
            Ordering::Less => println!("{}", "Too small!".red()),
            Ordering::Greater => println!("{}", "Too big!".red()),
            Ordering::Equal => {
                println!("{}", format!("You win in {attempts} attempt(s)!").green());
                break;
            }
        }
    }
}

// `::` is the path separator: it reaches items inside modules, and associated
// functions on types, e.g. `String::new()` or `Ordering::Less`.
