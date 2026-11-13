//! Book 9.3: use the type system to check values ONCE, at construction.
//! Run: cargo run -p error_handling --example validation_types

/// The Book's version: a guess between 1 and 100. `new` panics on bad input,
/// because the caller passing 0 or 101 is a BUG (a broken contract).
#[derive(Debug)]
pub struct Guess {
    value: i32, // private: nobody can make a Guess without going through new
}

impl Guess {
    pub fn new(value: i32) -> Guess {
        if !(1..=100).contains(&value) {
            panic!("Guess value must be between 1 and 100, got {value}.");
        }
        Guess { value }
    }

    pub fn value(&self) -> i32 {
        self.value
    }
}

/// The version for input that can legitimately be wrong (user typing):
/// return a Result and let the caller decide.
impl Guess {
    pub fn parse(input: &str) -> Result<Guess, String> {
        let value: i32 = input
            .trim()
            .parse()
            .map_err(|_| format!("{input:?} is not a number"))?;
        if (1..=100).contains(&value) {
            Ok(Guess { value })
        } else {
            Err(format!("{value} is not between 1 and 100"))
        }
    }
}

/// Functions that take a Guess never need to check the range again.
fn is_close(guess: &Guess, secret: i32) -> bool {
    (guess.value() - secret).abs() <= 5
}

fn main() {
    let g = Guess::new(42);
    println!("{g:?}, close to 45? {}", is_close(&g, 45));

    for input in ["50", " 7 ", "0", "abc"] {
        match Guess::parse(input) {
            Ok(guess) => println!("{input:?} -> {}", guess.value()),
            Err(why) => println!("{input:?} -> rejected: {why}"),
        }
    }

    // When YOU know more than the compiler: this literal is a valid IP,
    // so the Err case cannot happen. `expect` documents why.
    use std::net::IpAddr;
    let home: IpAddr = "127.0.0.1"
        .parse()
        .expect("Hardcoded IP address should be valid");
    println!("home = {home}");

    // Guess::new(200); // panics: Guess value must be between 1 and 100, got 200.
}
