//! Exercise 6: turn text into an enum, or `None` if it makes no sense.
//!
//! A tiny text-adventure parser. Accepted input (any amount of whitespace):
//!   "look"            -> Command::Look
//!   "go <direction>"  -> Command::Go(direction)   (north, east, south, west)
//!   "take <item>"     -> Command::Take(item)
//!   anything else     -> None
//!
//! Hints: `split_whitespace()` and `.next()` give you `Option<&str>` values.
//! `let ... else` and `?` keep the code flat.
//!
//! Test: cargo test -p enums_and_pattern_matching --features exercises --bin ex06_parse_command

#![allow(unused_variables)]

#[derive(Debug, Clone, Copy, PartialEq)]
enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Debug, PartialEq)]
enum Command {
    Look,
    Go(Direction),
    Take(String),
}

fn parse_direction(word: &str) -> Option<Direction> {
    todo!()
}

fn parse(input: &str) -> Option<Command> {
    todo!()
}

fn main() {
    for line in ["look", "go north", "take  lamp", "go up", "dance", ""] {
        println!("{line:?} -> {:?}", parse(line));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_directions() {
        assert_eq!(parse_direction("north"), Some(Direction::North));
        assert_eq!(parse_direction("west"), Some(Direction::West));
        assert_eq!(parse_direction("up"), None);
    }

    #[test]
    fn parses_valid_commands() {
        assert_eq!(parse("look"), Some(Command::Look));
        assert_eq!(parse("  go   south "), Some(Command::Go(Direction::South)));
        assert_eq!(parse("take lamp"), Some(Command::Take("lamp".into())));
    }

    #[test]
    fn rejects_nonsense() {
        assert_eq!(parse(""), None);
        assert_eq!(parse("dance"), None);
        assert_eq!(parse("go up"), None);
        assert_eq!(parse("go"), None);
        assert_eq!(parse("take"), None);
        assert_eq!(parse("look around"), None);
        assert_eq!(parse("go north now"), None);
    }
}
