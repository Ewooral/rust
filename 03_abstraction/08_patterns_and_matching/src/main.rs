//! Mini project for Book chapter 19: a toy robot that follows a script.
//!
//! The library (`src/lib.rs`) parses each line into a `Command` and runs it on
//! a `Robot`, using every kind of pattern from the chapter. This file just
//! picks a script, runs it and prints the report.
//!
//!   cargo run -p patterns_and_matching                  the built-in demo script
//!   cargo run -p patterns_and_matching -- my_script.txt your own script
//!   cargo test -p patterns_and_matching

use std::env;
use std::fs;
use std::process;

use patterns_and_matching::{Report, describe_position, run_script};

/// Mistakes on lines 6, 10, 13 and 18 are on purpose: they show the errors.
const DEMO: &str = "\
# The robot starts at home, facing north, with a blue pen.
move 3
right
paint red
f 4
fly to the moon
turn right
move 2
paint 128 128 128
move 99
left
undo
teleport 5
teleport -3 3
say hello from the north-west
paint white
move 2
paint purple
";

fn main() {
    // A match on an Option<String> from the iterator.
    let script = match env::args().nth(1) {
        Some(path) => fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("can't read {path}: {e}");
            process::exit(1);
        }),
        None => DEMO.to_string(),
    };

    let report = run_script(&script);
    print_report(&report);
}

fn print_report(
    Report {
        robot,
        events,
        errors,
    }: &Report,
) {
    // ^ a function parameter pattern destructuring a struct behind a reference.
    println!("Ran {} commands:", events.len());
    for (line, event) in events {
        println!("  line {line:>2}: {event}");
    }

    match errors.as_slice() {
        [] => println!("No mistakes in the script."),
        [(line, error)] => println!("1 line skipped: line {line}: {error}"),
        [..] => {
            println!("{} lines skipped:", errors.len());
            for (line, error) in errors {
                println!("  line {line:>2}: {error}");
            }
        }
    }

    println!();
    println!(
        "The robot is at {}, {}, facing {:?}. Map (H = home):",
        robot.position,
        describe_position(&robot.position),
        robot.facing
    );
    println!("{}", robot.map());
}
