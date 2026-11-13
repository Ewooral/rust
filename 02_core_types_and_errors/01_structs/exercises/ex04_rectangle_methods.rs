//! Exercise 4: implement the methods and associated function.
//!
//! The `self` parameters are given, except for `rotate`: choose that one yourself.
//!
//! Test: cargo test -p structs --features exercises --bin ex04_rectangle_methods

#![allow(unused_variables)]

#[derive(Debug, PartialEq)]
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    /// A square with sides of `size`.
    fn square(size: u32) -> Self {
        todo!()
    }

    /// width * height
    fn area(&self) -> u32 {
        todo!()
    }

    /// The distance all the way round.
    fn perimeter(&self) -> u32 {
        todo!()
    }

    /// True if `other` fits strictly inside `self` without rotating it.
    fn can_hold(&self, other: &Rectangle) -> bool {
        todo!()
    }

    /// Swap width and height. Add the right `self` parameter yourself.
    fn rotate(/* ? */) {
        todo!()
    }
}

fn main() {
    let r = Rectangle {
        width: 3,
        height: 4,
    };
    println!("area {}, perimeter {}", r.area(), r.perimeter());

    let mut tall = Rectangle {
        width: 2,
        height: 7,
    };
    println!(
        "square holds tall? {}",
        Rectangle::square(8).can_hold(&tall)
    );
    tall.rotate();
    println!("rotated: {tall:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_has_equal_sides() {
        assert_eq!(
            Rectangle::square(5),
            Rectangle {
                width: 5,
                height: 5
            }
        );
    }

    #[test]
    fn area_and_perimeter() {
        let r = Rectangle {
            width: 3,
            height: 4,
        };
        assert_eq!(r.area(), 12);
        assert_eq!(r.perimeter(), 14);
    }

    #[test]
    fn can_hold_is_strict() {
        let big = Rectangle::square(10);
        assert!(big.can_hold(&Rectangle::square(9)));
        assert!(!big.can_hold(&Rectangle::square(10)));
        assert!(!Rectangle::square(9).can_hold(&big));
    }

    #[test]
    fn rotate_swaps_sides() {
        let mut r = Rectangle {
            width: 2,
            height: 7,
        };
        r.rotate();
        assert_eq!(
            r,
            Rectangle {
                width: 7,
                height: 2
            }
        );
    }
}
