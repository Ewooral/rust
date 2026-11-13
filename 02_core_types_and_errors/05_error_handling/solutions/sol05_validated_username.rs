//! Solution to exercise 5: a type that can only hold valid values (Book 9.3).
//!
//! A `Username` is 3 to 16 characters, only ASCII letters, digits or `_`,
//! and does not start with a digit. Implement `Username::new`, checking the
//! rules in this order and returning the FIRST rule that fails.
//! Once a Username exists, it is valid: no other code ever re-checks it.
//!
//! Test: cargo test -p error_handling --features solutions --bin sol05_validated_username

#[derive(Debug, PartialEq)]
enum UsernameError {
    TooShort,
    TooLong,
    InvalidChar(char),
    StartsWithDigit,
}

#[derive(Debug, PartialEq)]
struct Username(String); // private field: only `new` can build one

impl Username {
    fn new(name: &str) -> Result<Username, UsernameError> {
        let length = name.chars().count(); // characters, not bytes
        if length < 3 {
            return Err(UsernameError::TooShort);
        }
        if length > 16 {
            return Err(UsernameError::TooLong);
        }
        if let Some(bad) = name
            .chars()
            .find(|c| !(c.is_ascii_alphanumeric() || *c == '_'))
        {
            return Err(UsernameError::InvalidChar(bad));
        }
        if name.starts_with(|c: char| c.is_ascii_digit()) {
            return Err(UsernameError::StartsWithDigit);
        }
        Ok(Username(name.to_string()))
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

fn greet(user: &Username) -> String {
    format!("Welcome, {}!", user.as_str())
}

fn main() {
    for name in ["ferris_99", "ab", "9lives", "hello world"] {
        match Username::new(name) {
            Ok(user) => println!("{}", greet(&user)),
            Err(e) => println!("{name:?} rejected: {e:?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_names() {
        assert_eq!(Username::new("ferris").unwrap().as_str(), "ferris");
        assert!(Username::new("a_1").is_ok());
        assert!(Username::new("sixteen_chars_ok").is_ok());
    }

    #[test]
    fn rejects_bad_lengths() {
        assert_eq!(Username::new("ab"), Err(UsernameError::TooShort));
        assert_eq!(
            Username::new("seventeen_chars_x"),
            Err(UsernameError::TooLong)
        );
    }

    #[test]
    fn rejects_bad_characters_and_leading_digit() {
        assert_eq!(
            Username::new("hi there"),
            Err(UsernameError::InvalidChar(' '))
        );
        assert_eq!(Username::new("café"), Err(UsernameError::InvalidChar('é')));
        assert_eq!(Username::new("9lives"), Err(UsernameError::StartsWithDigit));
    }
}
