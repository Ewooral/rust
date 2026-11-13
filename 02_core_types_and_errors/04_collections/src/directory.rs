//! The Book's chapter 8 project: a company directory.
//!
//! "Using a hash map and vectors, create a text interface to allow a user to
//! add employee names to a department in a company; for example, 'Add Sally
//! to Engineering' or 'Add Amir to Sales.' Then let the user retrieve a list
//! of all people in a department or all people in the company by department,
//! sorted alphabetically."

use std::collections::HashMap;

/// What the user typed, understood.
#[derive(Debug, PartialEq)]
pub enum Command {
    /// "Add <name> to <department>"
    Add { name: String, department: String },
    /// "List <department>"
    List(String),
    /// "List all"
    ListAll,
}

impl Command {
    /// Parses one line of text. Returns `None` if it is not a valid command.
    /// Words are case-insensitive ("add", "ADD"), names keep their case.
    ///
    /// ```
    /// use collections::directory::Command;
    /// assert_eq!(
    ///     Command::parse("Add Sally to Engineering"),
    ///     Some(Command::Add { name: "Sally".into(), department: "Engineering".into() })
    /// );
    /// assert_eq!(Command::parse("List all"), Some(Command::ListAll));
    /// assert_eq!(Command::parse("Fire everyone"), None);
    /// ```
    pub fn parse(line: &str) -> Option<Command> {
        let words: Vec<&str> = line.split_whitespace().collect();
        let (verb, rest) = words.split_first()?;
        match verb.to_lowercase().as_str() {
            "add" => {
                // Everything before "to" is the name, everything after it the
                // department, so "Add Mary Jane to Human Resources" works.
                let to = rest.iter().position(|w| w.eq_ignore_ascii_case("to"))?;
                let (name, department) = (&rest[..to], &rest[to + 1..]);
                if name.is_empty() || department.is_empty() {
                    return None;
                }
                Some(Command::Add {
                    name: name.join(" "),
                    department: department.join(" "),
                })
            }
            "list" if rest.len() == 1 && rest[0].eq_ignore_ascii_case("all") => {
                Some(Command::ListAll)
            }
            "list" if !rest.is_empty() => Some(Command::List(rest.join(" "))),
            _ => None,
        }
    }
}

/// Department name -> the people in it.
#[derive(Debug, Default)]
pub struct Directory {
    departments: HashMap<String, Vec<String>>,
}

impl Directory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a person. The entry API creates the department's Vec on first use.
    pub fn add(&mut self, name: &str, department: &str) {
        self.departments
            .entry(department.to_string())
            .or_default()
            .push(name.to_string());
    }

    /// The people in one department, sorted. Empty if it does not exist.
    pub fn people_in(&self, department: &str) -> Vec<String> {
        let mut people = self
            .departments
            .get(department)
            .cloned()
            .unwrap_or_default();
        people.sort();
        people
    }

    /// Every department (sorted) with its people (sorted).
    /// A HashMap has no order, so we collect the keys and sort them.
    pub fn everyone(&self) -> Vec<(String, Vec<String>)> {
        let mut names: Vec<&String> = self.departments.keys().collect();
        names.sort();
        names
            .into_iter()
            .map(|dept| (dept.clone(), self.people_in(dept)))
            .collect()
    }

    /// Runs one command and returns the text to show the user.
    pub fn run(&mut self, command: Command) -> String {
        match command {
            Command::Add { name, department } => {
                self.add(&name, &department);
                format!("Added {name} to {department}.")
            }
            Command::List(department) => {
                let people = self.people_in(&department);
                if people.is_empty() {
                    format!("Nobody works in {department}.")
                } else {
                    format!("{department}: {}", people.join(", "))
                }
            }
            Command::ListAll => {
                let all = self.everyone();
                if all.is_empty() {
                    return String::from("The company is empty.");
                }
                all.iter()
                    .map(|(dept, people)| format!("{dept}: {}", people.join(", ")))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_multi_word_names_and_departments() {
        assert_eq!(
            Command::parse("add Mary Jane TO Human Resources"),
            Some(Command::Add {
                name: "Mary Jane".into(),
                department: "Human Resources".into()
            })
        );
        assert_eq!(
            Command::parse("List Sales"),
            Some(Command::List("Sales".into()))
        );
    }

    #[test]
    fn rejects_incomplete_commands() {
        for bad in [
            "",
            "Add",
            "Add Sally",
            "Add to Sales",
            "Add Sally to",
            "List",
        ] {
            assert_eq!(Command::parse(bad), None, "{bad:?} should not parse");
        }
    }

    #[test]
    fn lists_are_sorted() {
        let mut d = Directory::new();
        d.add("Sally", "Engineering");
        d.add("Amir", "Sales");
        d.add("Bola", "Engineering");
        assert_eq!(d.people_in("Engineering"), ["Bola", "Sally"]);
        assert!(d.people_in("Marketing").is_empty());
        let depts: Vec<String> = d.everyone().into_iter().map(|(dept, _)| dept).collect();
        assert_eq!(depts, ["Engineering", "Sales"]);
    }

    #[test]
    fn run_reports_results() {
        let mut d = Directory::new();
        assert_eq!(d.run(Command::ListAll), "The company is empty.");
        assert_eq!(
            d.run(Command::parse("Add Sally to Engineering").unwrap()),
            "Added Sally to Engineering."
        );
        assert_eq!(
            d.run(Command::List("Sales".into())),
            "Nobody works in Sales."
        );
        assert_eq!(d.run(Command::ListAll), "Engineering: Sally");
    }
}
