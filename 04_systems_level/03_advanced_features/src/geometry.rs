//! Points and trails: operator overloading, an iterator with an associated
//! type, function-pointer and closure parameters, and a supertrait.

use std::fmt;
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub};
use std::thread;

use crate::OutlinePrint;
use crate::raw;
use crate::transform::Transform;
use crate::units::Meters;

/// A 2D vector (or point), measured in metres.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2::new(0.0, 0.0);

    pub const fn new(x: f64, y: f64) -> Vec2 {
        Vec2 { x, y }
    }

    /// The straight-line length, as a typed distance.
    pub fn length(self) -> Meters {
        Meters(self.x.hypot(self.y))
    }

    /// Mirror in the y axis. Ordinary methods like this one can be used as
    /// FUNCTION POINTERS: `Vec2::flip_x` has the type `fn(Vec2) -> Vec2`.
    pub fn flip_x(self) -> Vec2 {
        Vec2::new(-self.x, self.y)
    }

    pub fn flip_y(self) -> Vec2 {
        Vec2::new(self.x, -self.y)
    }

    /// A quarter turn anticlockwise.
    pub fn rotate90(self) -> Vec2 {
        Vec2::new(-self.y, self.x)
    }
}

// ---- Operator overloading: each operator is a trait in std::ops ----

impl Add for Vec2 {
    type Output = Vec2; // the ASSOCIATED TYPE: what `a + b` produces
    fn add(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x - other.x, self.y - other.y)
    }
}

impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2::new(-self.x, -self.y)
    }
}

/// `vector * 2.0`
impl Mul<f64> for Vec2 {
    type Output = Vec2;
    fn mul(self, factor: f64) -> Vec2 {
        Vec2::new(self.x * factor, self.y * factor)
    }
}

/// `2.0 * vector`. Both `Mul` and `f64` are foreign, yet this is allowed by
/// the orphan rule because OUR type `Vec2` appears as the type parameter.
impl Mul<Vec2> for f64 {
    type Output = Vec2;
    fn mul(self, v: Vec2) -> Vec2 {
        v * self
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, other: Vec2) {
        *self = *self + other;
    }
}

impl MulAssign<f64> for Vec2 {
    fn mul_assign(&mut self, factor: f64) {
        *self = *self * factor;
    }
}

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

/// A trail: points visited in order. Build one with the `trail!` macro.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Trail {
    points: Vec<Vec2>,
}

impl Trail {
    pub fn new() -> Trail {
        Trail { points: Vec::new() }
    }

    pub fn push(&mut self, point: Vec2) {
        self.points.push(point);
    }

    pub fn points(&self) -> &[Vec2] {
        &self.points
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Each leg of the trail as a `(from, to)` pair.
    pub fn segments(&self) -> Segments<'_> {
        Segments { rest: &self.points }
    }

    /// Total distance walked. `.sum()` works because `Meters: Sum`.
    pub fn length(&self) -> Meters {
        self.segments().map(|(from, to)| (to - from).length()).sum()
    }

    /// Moves every point with a FUNCTION POINTER, e.g. `Vec2::rotate90`.
    pub fn apply(&mut self, transform: Transform) {
        for point in &mut self.points {
            *point = transform(*point);
        }
    }

    /// Moves every point with any closure (or function: `fn` pointers
    /// implement `Fn` too).
    pub fn apply_with(&mut self, transform: impl Fn(Vec2) -> Vec2) {
        for point in &mut self.points {
            *point = transform(*point);
        }
    }

    /// Scales the points on two threads at once. Each thread needs its OWN
    /// `&mut` half of the list; `raw::split_at_mut` (an audited `unsafe`
    /// block behind a safe function) provides the two halves.
    pub fn scale_in_parallel(&mut self, factor: f64) {
        let mid = self.points.len() / 2;
        let (left, right) = raw::split_at_mut(&mut self.points, mid);
        thread::scope(|s| {
            s.spawn(|| left.iter_mut().for_each(|p| *p *= factor));
            right.iter_mut().for_each(|p| *p *= factor);
        });
    }
}

impl FromIterator<Vec2> for Trail {
    fn from_iter<I: IntoIterator<Item = Vec2>>(iter: I) -> Trail {
        Trail {
            points: iter.into_iter().collect(),
        }
    }
}

impl fmt::Display for Trail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, point) in self.points.iter().enumerate() {
            if i > 0 {
                write!(f, " -> ")?;
            }
            write!(f, "{point}")?;
        }
        Ok(())
    }
}

/// `Trail: Display`, so it may use the `OutlinePrint` supertrait's defaults.
impl OutlinePrint for Trail {}

/// The legs of a trail. An iterator with an ASSOCIATED TYPE: an iterator
/// type has exactly one `Item`, so callers never have to name it.
pub struct Segments<'a> {
    rest: &'a [Vec2],
}

impl Iterator for Segments<'_> {
    type Item = (Vec2, Vec2);

    fn next(&mut self) -> Option<(Vec2, Vec2)> {
        match self.rest {
            [from, to, ..] => {
                let leg = (*from, *to);
                self.rest = &self.rest[1..];
                Some(leg)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trail;

    #[test]
    fn vector_operators() {
        let a = Vec2::new(1.0, 2.0);
        let b = Vec2::new(3.0, -1.0);
        assert_eq!(a + b, Vec2::new(4.0, 1.0));
        assert_eq!(a - b, Vec2::new(-2.0, 3.0));
        assert_eq!(-a, Vec2::new(-1.0, -2.0));
        assert_eq!(a * 2.0, 2.0 * a);
        let mut c = a;
        c += b;
        c *= 10.0;
        assert_eq!(c, Vec2::new(40.0, 10.0));
    }

    #[test]
    fn segments_and_length() {
        let walk = trail![(0, 0), (3, 4), (3, 10)];
        let legs: Vec<_> = walk.segments().collect();
        assert_eq!(legs.len(), 2);
        assert_eq!(legs[1], (Vec2::new(3.0, 4.0), Vec2::new(3.0, 10.0)));
        assert_eq!(walk.length(), Meters(11.0));
        assert_eq!(trail![(5, 5)].length(), Meters(0.0));
    }

    #[test]
    fn apply_takes_fn_pointers_and_closures() {
        let mut walk = trail![(1, 0), (0, 2)];
        walk.apply(Vec2::rotate90);
        assert_eq!(walk.points(), [Vec2::new(0.0, 1.0), Vec2::new(-2.0, 0.0)]);
        let shift = Vec2::new(1.0, 1.0);
        walk.apply_with(|p| p + shift);
        walk.apply_with(Vec2::flip_y); // a function where a closure is expected
        assert_eq!(walk.points(), [Vec2::new(1.0, -2.0), Vec2::new(-1.0, -1.0)]);
    }

    #[test]
    fn scale_in_parallel_touches_every_point_once() {
        for n in [0, 1, 2, 7] {
            let mut walk: Trail = (0..n).map(|i| Vec2::new(f64::from(i), 1.0)).collect();
            walk.scale_in_parallel(3.0);
            let expected: Vec<Vec2> = (0..n).map(|i| Vec2::new(f64::from(i) * 3.0, 3.0)).collect();
            assert_eq!(walk.points(), expected);
        }
    }

    #[test]
    fn display_and_outline() {
        let walk = trail![(0, 0), (1.5, 2)];
        assert_eq!(walk.to_string(), "(0, 0) -> (1.5, 2)");
        assert!(walk.outline().contains("* (0, 0) -> (1.5, 2) *"));
    }
}
