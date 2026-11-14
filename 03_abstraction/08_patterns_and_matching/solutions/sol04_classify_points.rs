//! Solution to exercise 4: destructuring, ranges, guards and `@` (Book 19.3).
//!
//! 1. `classify` uses ONE `match` on the point (no `if`), checked in this order:
//!    - the origin -> "origin"
//!    - on the x axis (y is 0) -> "x axis at {x}"
//!    - on the y axis (x is 0) -> "y axis at {y}"
//!    - on the diagonal (x == y) -> "diagonal at {x}" (use a match guard)
//!    - both x and y in 1..=9 -> "near ({x}, {y})" (use ranges)
//!    - anything else -> "far ({x}, {y})"
//!
//! 2. `grade` turns a score into a grade, keeping the score with `@`:
//!    90..=100 is "A (95)", 70..=89 is "B (75)", 50..=69 is "C (50)",
//!    0..=49 is "fail (12)", and above 100 is "invalid score 150".
//!    No `_` arm: make the ranges cover every u8.
//!
//! Test: cargo test -p patterns_and_matching --features solutions --bin sol04_classify_points

struct Point {
    x: i32,
    y: i32,
}

fn classify(point: &Point) -> String {
    // Matching a &Point: x and y are bound as &i32, so use *x in guards.
    // Arms are tried top to bottom, so the order IS the rule order.
    match point {
        Point { x: 0, y: 0 } => "origin".to_string(),
        Point { x, y: 0 } => format!("x axis at {x}"),
        Point { x: 0, y } => format!("y axis at {y}"),
        Point { x, y } if x == y => format!("diagonal at {x}"),
        Point {
            x: x @ 1..=9,
            y: y @ 1..=9,
        } => format!("near ({x}, {y})"),
        Point { x, y } => format!("far ({x}, {y})"),
    }
}

fn grade(score: u8) -> String {
    // `@` tests the range AND keeps the value. The five ranges cover
    // 0..=255 exactly, so no `_` is needed.
    match score {
        s @ 90..=100 => format!("A ({s})"),
        s @ 70..=89 => format!("B ({s})"),
        s @ 50..=69 => format!("C ({s})"),
        s @ 0..=49 => format!("fail ({s})"),
        s @ 101.. => format!("invalid score {s}"),
    }
}

fn main() {
    for (x, y) in [(0, 0), (4, 0), (0, -2), (3, 3), (2, 5), (20, 1)] {
        println!("({x}, {y}) -> {}", classify(&Point { x, y }));
    }
    for score in [95, 75, 50, 12, 150] {
        println!("{score} -> {}", grade(score));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(x: i32, y: i32) -> String {
        classify(&Point { x, y })
    }

    #[test]
    fn axes_and_origin() {
        assert_eq!(c(0, 0), "origin");
        assert_eq!(c(4, 0), "x axis at 4");
        assert_eq!(c(0, -2), "y axis at -2");
    }

    #[test]
    fn diagonal_near_and_far() {
        assert_eq!(c(3, 3), "diagonal at 3");
        assert_eq!(c(-7, -7), "diagonal at -7");
        assert_eq!(c(2, 5), "near (2, 5)");
        assert_eq!(c(9, 1), "near (9, 1)");
        assert_eq!(c(10, 1), "far (10, 1)");
        assert_eq!(c(-2, 5), "far (-2, 5)");
    }

    #[test]
    fn grades_keep_the_score() {
        assert_eq!(grade(100), "A (100)");
        assert_eq!(grade(90), "A (90)");
        assert_eq!(grade(89), "B (89)");
        assert_eq!(grade(50), "C (50)");
        assert_eq!(grade(0), "fail (0)");
        assert_eq!(grade(255), "invalid score 255");
    }
}
