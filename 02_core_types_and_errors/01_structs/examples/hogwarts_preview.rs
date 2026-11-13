//! Preview: where structs go next. Uses enums (next lesson) and custom errors
//! (lesson 05) to model a Hogwarts student. Read it again after those lessons.
//!
//! Run: cargo run -p structs --example hogwarts_preview
//! Test: cargo test -p structs --example hogwarts_preview

use std::error::Error;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum House {
    Gryffindor,
    Hufflepuff,
    Ravenclaw,
    Slytherin,
}

impl FromStr for House {
    type Err = StudentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Gryffindor" => Ok(House::Gryffindor),
            "Hufflepuff" => Ok(House::Hufflepuff),
            "Ravenclaw" => Ok(House::Ravenclaw),
            "Slytherin" => Ok(House::Slytherin),
            other => Err(StudentError::UnknownHouse(other.to_string())),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum StudentError {
    MissingName,
    UnknownHouse(String),
    InvalidGrade(u8),
}

impl fmt::Display for StudentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StudentError::MissingName => write!(f, "name must not be empty"),
            StudentError::UnknownHouse(h) => write!(f, "unknown house: {h}"),
            StudentError::InvalidGrade(g) => write!(f, "grade {g} is not between 0 and 100"),
        }
    }
}

impl Error for StudentError {}

#[derive(Debug)]
pub struct Student {
    name: String,
    house: House,
    patronus: String,
    grade: u8,
}

impl Student {
    const DEFAULT_GRADE: u8 = 89;

    pub fn new(name: &str, house: &str, patronus: &str) -> Result<Self, StudentError> {
        if name.trim().is_empty() {
            return Err(StudentError::MissingName);
        }
        Ok(Self {
            name: name.to_string(),
            house: house.parse()?,
            patronus: patronus.to_string(),
            grade: Self::DEFAULT_GRADE,
        })
    }

    /// Only a Stag or Otter patronus can cast the advanced charm.
    pub fn charm(&self) -> Option<&str> {
        match self.patronus.as_str() {
            "Stag" | "Otter" => Some(self.patronus.as_str()),
            _ => None,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn house(&self) -> House {
        self.house
    }

    pub fn grade(&self) -> u8 {
        self.grade
    }

    pub fn set_grade(&mut self, grade: u8) -> Result<(), StudentError> {
        if grade > 100 {
            return Err(StudentError::InvalidGrade(grade));
        }
        self.grade = grade;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut student = Student::new("Elijah", "Gryffindor", "Stag")?;
    println!("{student:?}");
    println!("charm: {:?}", student.charm());
    println!("{} is in {:?}", student.name(), student.house());
    student.set_grade(100)?;
    println!("grade: {}%", student.grade());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_a_valid_student() {
        let s = Student::new("Elijah", "Ravenclaw", "Otter").unwrap();
        assert_eq!(s.house(), House::Ravenclaw);
        assert_eq!(s.grade(), 89);
    }

    #[test]
    fn rejects_empty_name() {
        assert_eq!(
            Student::new("  ", "Gryffindor", "Stag").unwrap_err(),
            StudentError::MissingName
        );
    }

    #[test]
    fn rejects_unknown_house() {
        let err = Student::new("A", "Mission House", "Stag").unwrap_err();
        assert_eq!(err, StudentError::UnknownHouse("Mission House".into()));
    }

    #[test]
    fn charm_depends_on_patronus_not_name() {
        assert_eq!(
            Student::new("Stag", "Slytherin", "Cat").unwrap().charm(),
            None
        );
        assert_eq!(
            Student::new("Bob", "Slytherin", "Otter").unwrap().charm(),
            Some("Otter")
        );
    }

    #[test]
    fn grade_is_validated() {
        let mut s = Student::new("A", "Hufflepuff", "Cat").unwrap();
        assert_eq!(s.set_grade(101), Err(StudentError::InvalidGrade(101)));
        assert!(s.set_grade(100).is_ok());
        assert_eq!(s.grade(), 100);
    }
}
