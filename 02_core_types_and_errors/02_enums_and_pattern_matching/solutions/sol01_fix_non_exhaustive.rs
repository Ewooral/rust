//! Solution to exercise 1: make it compile (E0004).
//!
//! Someone added `West` to the enum and forgot the match. Read the error: it
//! names the missing pattern. Fix it by adding a real arm, NOT `_ =>`.
//! (Why is `_` a bad idea here? What happens next time someone adds a variant?)
//!
//! Run: cargo run -p enums_and_pattern_matching --features solutions --bin sol01_fix_non_exhaustive

#[derive(Debug, Clone, Copy)]
enum Direction {
    North,
    East,
    South,
    West,
}

fn turn_right(d: Direction) -> Direction {
    match d {
        Direction::North => Direction::East,
        Direction::East => Direction::South,
        Direction::South => Direction::West,
        Direction::West => Direction::North,
    }
}

fn main() {
    let mut facing = Direction::North;
    for _ in 0..4 {
        facing = turn_right(facing);
        println!("now facing {facing:?}");
    }
}
