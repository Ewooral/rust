//! Units of measure as NEWTYPES (Book 20.2 and 20.3).
//!
//! `Meters(5.0)` and `Seconds(5.0)` are both an `f64` inside, but they are
//! different types, so the compiler stops you adding a distance to a time.
//! Wrapping costs nothing at run time: a `Meters` is exactly 8 bytes.
//!
//! Each unit needs the same handful of impls, so a declarative macro writes
//! them (`unit!`). Only the impls that differ are written by hand below.

use std::fmt;
use std::iter::Sum;
use std::ops::{Add, Div, Mul, Sub};

/// Defines a unit newtype with `+`, `-`, `* f64`, `.sum()` and `Display`.
///
/// `$(#[$attr:meta])*` captures any doc comments or attributes written before
/// the name and pastes them back onto the struct.
macro_rules! unit {
    ($(#[$attr:meta])* $name:ident, $symbol:literal) => {
        $(#[$attr])*
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
        pub struct $name(pub f64);

        impl Add for $name {
            type Output = $name;
            fn add(self, other: $name) -> $name {
                $name(self.0 + other.0)
            }
        }

        impl Sub for $name {
            type Output = $name;
            fn sub(self, other: $name) -> $name {
                $name(self.0 - other.0)
            }
        }

        impl Mul<f64> for $name {
            type Output = $name;
            fn mul(self, factor: f64) -> $name {
                $name(self.0 * factor)
            }
        }

        impl Sum for $name {
            fn sum<I: Iterator<Item = $name>>(iter: I) -> $name {
                iter.fold($name(0.0), Add::add)
            }
        }

        /// Prints the number and the unit symbol. `{:.1}` sets the decimals (default 2).
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let decimals = f.precision().unwrap_or(2);
                write!(f, "{:.*} {}", decimals, self.0, $symbol)
            }
        }
    };
}

unit!(
    /// A distance in metres.
    Meters,
    "m"
);
unit!(
    /// A distance in millimetres.
    Millimeters,
    "mm"
);
unit!(
    /// A length of time in seconds.
    Seconds,
    "s"
);
unit!(
    /// A speed in metres per second.
    MetersPerSecond,
    "m/s"
);

/// The Book's example of a DEFAULT TYPE PARAMETER. `Add` is declared as
/// `trait Add<Rhs = Self>`, so `impl Add for Meters` means `Add<Meters>`.
/// Here we choose a different right-hand side: metres + millimetres.
impl Add<Millimeters> for Meters {
    type Output = Meters;
    fn add(self, other: Millimeters) -> Meters {
        Meters(self.0 + other.0 / 1000.0)
    }
}

impl From<Millimeters> for Meters {
    fn from(mm: Millimeters) -> Meters {
        Meters(mm.0 / 1000.0)
    }
}

/// distance / time = speed. The ASSOCIATED TYPE `Output` says what a
/// division produces, and here it is a third type.
impl Div<Seconds> for Meters {
    type Output = MetersPerSecond;
    fn div(self, time: Seconds) -> MetersPerSecond {
        MetersPerSecond(self.0 / time.0)
    }
}

/// distance / speed = time.
impl Div<MetersPerSecond> for Meters {
    type Output = Seconds;
    fn div(self, speed: MetersPerSecond) -> Seconds {
        Seconds(self.0 / speed.0)
    }
}

/// speed * time = distance.
impl Mul<Seconds> for MetersPerSecond {
    type Output = Meters;
    fn mul(self, time: Seconds) -> Meters {
        Meters(self.0 * time.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_units_add_and_subtract() {
        assert_eq!(Meters(1.5) + Meters(2.0), Meters(3.5));
        assert_eq!(Seconds(10.0) - Seconds(4.0), Seconds(6.0));
        assert_eq!(Meters(2.0) * 3.0, Meters(6.0));
    }

    #[test]
    fn metres_plus_millimetres_uses_the_rhs_type_parameter() {
        assert_eq!(Meters(1.0) + Millimeters(250.0), Meters(1.25));
        assert_eq!(Meters::from(Millimeters(1500.0)), Meters(1.5));
    }

    #[test]
    fn division_and_multiplication_change_the_unit() {
        let speed = Meters(100.0) / Seconds(8.0);
        assert_eq!(speed, MetersPerSecond(12.5));
        assert_eq!(speed * Seconds(2.0), Meters(25.0));
        assert_eq!(Meters(50.0) / MetersPerSecond(12.5), Seconds(4.0));
    }

    #[test]
    fn sum_and_display() {
        let legs = [Meters(1.0), Meters(2.5), Meters(0.25)];
        let total: Meters = legs.into_iter().sum();
        assert_eq!(total.to_string(), "3.75 m");
        assert_eq!(format!("{:.1}", MetersPerSecond(4.1666)), "4.2 m/s");
    }

    #[test]
    fn a_newtype_is_free() {
        assert_eq!(size_of::<Meters>(), size_of::<f64>());
    }
}
