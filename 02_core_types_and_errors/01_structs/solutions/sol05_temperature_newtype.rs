//! Solution to exercise 5 (Beyond the Book): newtypes stop you mixing units.
//!
//! Implement the conversions. F = C * 9 / 5 + 32, and C = (F - 32) * 5 / 9.
//! Then uncomment the last line of `main` and read the error: that mistake is
//! now impossible.
//!
//! Test: cargo test -p structs --features solutions --bin sol05_temperature_newtype

#[derive(Debug, Clone, Copy, PartialEq)]
struct Celsius(f64);

#[derive(Debug, Clone, Copy, PartialEq)]
struct Fahrenheit(f64);

impl Celsius {
    fn to_fahrenheit(self) -> Fahrenheit {
        Fahrenheit(self.0 * 9.0 / 5.0 + 32.0)
    }
}

impl Fahrenheit {
    fn to_celsius(self) -> Celsius {
        Celsius((self.0 - 32.0) * 5.0 / 9.0)
    }
}

fn is_fever(t: Celsius) -> bool {
    t.0 >= 38.0
}

fn main() {
    let body = Celsius(37.0);
    println!("{body:?} = {:?}", body.to_fahrenheit());
    println!("fever? {}", is_fever(body));
    println!("100 F = {:?}", Fahrenheit(100.0).to_celsius());
    // println!("fever? {}", is_fever(Fahrenheit(100.0)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn celsius_to_fahrenheit() {
        assert_eq!(Celsius(100.0).to_fahrenheit(), Fahrenheit(212.0));
        assert_eq!(Celsius(0.0).to_fahrenheit(), Fahrenheit(32.0));
        assert_eq!(Celsius(-40.0).to_fahrenheit(), Fahrenheit(-40.0));
    }

    #[test]
    fn fahrenheit_to_celsius() {
        assert_eq!(Fahrenheit(212.0).to_celsius(), Celsius(100.0));
        assert_eq!(Fahrenheit(32.0).to_celsius(), Celsius(0.0));
    }
}
