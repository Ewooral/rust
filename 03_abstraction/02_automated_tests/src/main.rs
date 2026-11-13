//! Mini project for Book chapter 11: a bowling scorer.
//!
//! This binary is deliberately thin. All the logic, and therefore all the
//! things worth testing, lives in the library (src/bowling.rs), which unit
//! tests, integration tests, doc tests and property tests all exercise.
//!
//!   cargo run -p automated_tests                         a sample game
//!   cargo run -p automated_tests -- 10 10 10 10 10 10 10 10 10 10 10 10
//!   cargo test -p automated_tests                        every kind of test

use automated_tests::bowling::score;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rolls: Vec<u32> = if args.is_empty() {
        println!("(no rolls given, scoring a sample game)");
        vec![10, 7, 3, 9, 0, 10, 0, 8, 8, 2, 0, 6, 10, 10, 10, 8, 1]
    } else {
        match args.iter().map(|a| a.parse()).collect() {
            Ok(rolls) => rolls,
            Err(e) => {
                eprintln!("every roll must be a whole number: {e}");
                std::process::exit(2);
            }
        }
    };

    match score(&rolls) {
        Ok(total) => println!("rolls {rolls:?}\nscore: {total}"),
        Err(e) => {
            eprintln!("not a valid game: {e}");
            std::process::exit(1);
        }
    }
}
