//! Solution to exercise 5: variants that hold different data.
//!
//! Implement `area` and `name` with a `match` that binds each variant's data.
//!
//! Test: cargo test -p enums_and_pattern_matching --features solutions --bin sol05_shapes

use std::f64::consts::PI;

#[derive(Debug)]
enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    Square(f64),
}

impl Shape {
    fn area(&self) -> f64 {
        match self {
            Shape::Circle { radius } => PI * radius * radius,
            Shape::Rectangle { width, height } => width * height,
            Shape::Square(side) => side * side,
        }
    }

    /// "circle", "rectangle" or "square"
    fn name(&self) -> &'static str {
        match self {
            Shape::Circle { .. } => "circle",
            Shape::Rectangle { .. } => "rectangle",
            Shape::Square(_) => "square",
        }
    }
}

fn total_area(shapes: &[Shape]) -> f64 {
    shapes.iter().map(Shape::area).sum()
}

fn main() {
    let shapes = [
        Shape::Circle { radius: 1.0 },
        Shape::Rectangle {
            width: 2.0,
            height: 3.0,
        },
        Shape::Square(4.0),
    ];
    for s in &shapes {
        println!("{} has area {:.2}", s.name(), s.area());
    }
    println!("total: {:.2}", total_area(&shapes));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn areas() {
        assert!(close(Shape::Circle { radius: 2.0 }.area(), 4.0 * PI));
        assert!(close(
            Shape::Rectangle {
                width: 2.0,
                height: 3.5
            }
            .area(),
            7.0
        ));
        assert!(close(Shape::Square(3.0).area(), 9.0));
    }

    #[test]
    fn names() {
        assert_eq!(Shape::Circle { radius: 1.0 }.name(), "circle");
        assert_eq!(
            Shape::Rectangle {
                width: 1.0,
                height: 1.0
            }
            .name(),
            "rectangle"
        );
        assert_eq!(Shape::Square(1.0).name(), "square");
    }
}
