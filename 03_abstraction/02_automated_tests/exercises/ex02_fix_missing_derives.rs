//! Exercise 2: make the tests compile (E0369, E0277).
//!
//! `assert_eq!` needs to COMPARE two values (==) and to PRINT them if they
//! differ ({:?}). Add the two derives that make that possible.
//!
//! Test: cargo test -p automated_tests --features exercises --bin ex02_fix_missing_derives

struct Point {
    x: i32,
    y: i32,
}

fn mirror(p: Point) -> Point {
    Point { x: -p.x, y: p.y }
}

fn main() {
    let p = mirror(Point { x: 3, y: 4 });
    println!("({}, {})", p.x, p.y);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mirror_flips_x() {
        assert_eq!(mirror(Point { x: 3, y: 4 }), Point { x: -3, y: 4 });
    }
}
