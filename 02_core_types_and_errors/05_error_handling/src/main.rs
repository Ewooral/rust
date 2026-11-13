//! Mini project for Book chapter 9: a grade-book loader that never crashes on
//! bad data.
//!
//! It reads a CSV file of students. A problem with the FILE (it is missing)
//! stops the program with an error message. A problem with one LINE (a bad
//! grade, an unknown house) is reported with its line number, and the rest
//! of the file is still loaded.
//!
//!   cargo run -p error_handling                    uses data/students.csv
//!   cargo run -p error_handling -- other.csv       your own file
//!   cargo run -p error_handling -- missing.csv     see a fatal error
//!   cargo test -p error_handling

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::num::ParseIntError;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum House {
    Gryffindor,
    Hufflepuff,
    Ravenclaw,
    Slytherin,
}

impl FromStr for House {
    type Err = LineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Gryffindor" => Ok(House::Gryffindor),
            "Hufflepuff" => Ok(House::Hufflepuff),
            "Ravenclaw" => Ok(House::Ravenclaw),
            "Slytherin" => Ok(House::Slytherin),
            other => Err(LineError::UnknownHouse(other.to_string())),
        }
    }
}

/// Everything that can be wrong with ONE line of the file.
#[derive(Debug, PartialEq)]
enum LineError {
    WrongFieldCount(usize),
    MissingName,
    UnknownHouse(String),
    BadGrade(ParseIntError),
    GradeOutOfRange(u8),
}

impl fmt::Display for LineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LineError::WrongFieldCount(n) => write!(f, "expected 3 fields, found {n}"),
            LineError::MissingName => write!(f, "the name is empty"),
            LineError::UnknownHouse(h) => write!(f, "unknown house {h:?}"),
            LineError::BadGrade(e) => write!(f, "the grade is not a number ({e})"),
            LineError::GradeOutOfRange(g) => write!(f, "grade {g} is above 100"),
        }
    }
}

impl Error for LineError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            LineError::BadGrade(e) => Some(e),
            _ => None,
        }
    }
}

/// Lets `?` turn a ParseIntError into a LineError.
impl From<ParseIntError> for LineError {
    fn from(e: ParseIntError) -> Self {
        LineError::BadGrade(e)
    }
}

#[derive(Debug, PartialEq)]
struct Student {
    name: String,
    house: House,
    grade: u8,
}

/// Parses one CSV line. Every failure is a value we return, never a panic.
fn parse_line(line: &str) -> Result<Student, LineError> {
    let fields: Vec<&str> = line.split(',').map(str::trim).collect();
    let &[name, house, grade] = fields.as_slice() else {
        return Err(LineError::WrongFieldCount(fields.len()));
    };
    if name.is_empty() {
        return Err(LineError::MissingName);
    }
    let house: House = house.parse()?; // our FromStr returns LineError already
    let grade: u8 = grade.parse()?; // ParseIntError -> LineError via From
    if grade > 100 {
        return Err(LineError::GradeOutOfRange(grade));
    }
    Ok(Student {
        name: name.to_string(),
        house,
        grade,
    })
}

/// The good students, and the (line number, error) of every bad line.
fn parse_file(text: &str) -> (Vec<Student>, Vec<(usize, LineError)>) {
    let mut students = Vec::new();
    let mut problems = Vec::new();
    // skip(1) skips the header; line numbers count from 1 like an editor's.
    for (index, line) in text.lines().enumerate().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        match parse_line(line) {
            Ok(student) => students.push(student),
            Err(e) => problems.push((index + 1, e)),
        }
    }
    (students, problems)
}

fn house_averages(students: &[Student]) -> BTreeMap<House, f64> {
    let mut totals: BTreeMap<House, (u32, u32)> = BTreeMap::new();
    for s in students {
        let entry = totals.entry(s.house).or_insert((0, 0));
        entry.0 += u32::from(s.grade);
        entry.1 += 1;
    }
    totals
        .into_iter()
        .map(|(house, (sum, count))| (house, f64::from(sum) / f64::from(count)))
        .collect()
}

/// `main` returns a Result: a missing file ends the program with a message
/// and a non-zero exit code, instead of a panic.
fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            // CARGO_MANIFEST_DIR is this crate's folder, set by Cargo at build time.
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("data")
                .join("students.csv")
        });

    let text =
        fs::read_to_string(&path).map_err(|e| format!("could not read {}: {e}", path.display()))?;

    let (students, problems) = parse_file(&text);

    println!("loaded {} students from {}", students.len(), path.display());
    for (line, error) in &problems {
        println!("  line {line}: skipped, {error}");
    }

    println!("house averages:");
    for (house, average) in house_averages(&students) {
        println!("  {house:<11?} {average:.1}");
    }

    if let Some(top) = students.iter().max_by_key(|s| s.grade) {
        println!("top student: {} ({}%)", top.name, top.grade);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_good_line() {
        assert_eq!(
            parse_line(" Luna , Ravenclaw , 88 "),
            Ok(Student {
                name: "Luna".into(),
                house: House::Ravenclaw,
                grade: 88
            })
        );
    }

    #[test]
    fn each_bad_line_gets_its_own_error() {
        assert_eq!(parse_line("a,b"), Err(LineError::WrongFieldCount(2)));
        assert_eq!(parse_line(",Slytherin,60"), Err(LineError::MissingName));
        assert_eq!(
            parse_line("P,Poltergeist,50"),
            Err(LineError::UnknownHouse("Poltergeist".into()))
        );
        assert!(matches!(
            parse_line("N,Gryffindor,seventy"),
            Err(LineError::BadGrade(_))
        ));
        assert_eq!(
            parse_line("E,Hufflepuff,105"),
            Err(LineError::GradeOutOfRange(105))
        );
    }

    #[test]
    fn a_bad_grade_keeps_its_cause() {
        let err = parse_line("N,Gryffindor,300").unwrap_err();
        // 300 does not fit in a u8, so this is a parse error, not a range error.
        assert!(err.source().is_some());
    }

    #[test]
    fn the_sample_file_loads_good_lines_and_reports_bad_ones() {
        let text = include_str!("../data/students.csv");
        let (students, problems) = parse_file(text);
        assert_eq!(students.len(), 6);
        let lines: Vec<usize> = problems.iter().map(|(line, _)| *line).collect();
        assert_eq!(lines, [7, 8, 9, 10, 11, 12]);
    }

    #[test]
    fn averages_per_house() {
        let (students, _) = parse_file("header\nA,Gryffindor,80\nB,Gryffindor,90\nC,Slytherin,70");
        let averages = house_averages(&students);
        assert_eq!(averages[&House::Gryffindor], 85.0);
        assert_eq!(averages[&House::Slytherin], 70.0);
    }
}
