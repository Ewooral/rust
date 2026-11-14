//! Exercise 5: a consuming builder with an error enum (pass 4 tests).
//!
//! `ReportBuilder` collects a title, columns and rows, and `build` checks
//! everything at once and returns `Result<Report, ReportError>`. Implement
//! each `todo!()`:
//!
//!   title(t), column(c)   store them (`impl Into<String>`: callers may pass
//!                         `&str` or `String`)
//!   row(cells)            store one row. `cells` is any `IntoIterator` of
//!                         anything `Into<String>`: an array of `&str`, a
//!                         `Vec<String>`, a `map(…)`.
//!   max_rows(n)           keep at most `n` rows (the default is 100)
//!   build()               in this order:
//!                         - no title                -> MissingTitle
//!                         - no columns              -> NoColumns
//!                         - max_rows set to 0       -> ZeroMaxRows
//!                         - a row with the wrong number of cells ->
//!                           WrongWidth { row (counting from 1), found, expected }
//!                         - otherwise a Report, with only the first
//!                           `max_rows` rows.
//!
//! Each method takes `self` and returns `Self`, so calls chain:
//! `ReportBuilder::new().title("x").column("a").build()`.
//!
//! Test: cargo test -p api_design --features exercises --bin ex05_builder_with_errors

#![allow(unused_variables, dead_code, unused_mut)]

/// A finished table, ready to print.
#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    title: String,
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Report {
    /// The title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// The rows, each a slice of cells.
    pub fn rows(&self) -> impl Iterator<Item = &[String]> + '_ {
        self.rows.iter().map(Vec::as_slice)
    }

    /// The report as text, columns separated by ` | `.
    pub fn render(&self) -> String {
        let mut text = format!("{}\n{}\n", self.title, self.columns.join(" | "));
        for row in &self.rows {
            text += &row.join(" | ");
            text.push('\n');
        }
        text
    }
}

/// Why a report couldn't be built.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ReportError {
    #[error("a report needs a title")]
    MissingTitle,
    #[error("a report needs at least one column")]
    NoColumns,
    #[error("max_rows must be at least 1")]
    ZeroMaxRows,
    #[error("row {row} has {found} cells, but there are {expected} columns")]
    WrongWidth {
        row: usize,
        found: usize,
        expected: usize,
    },
}

/// Builds a [`Report`].
#[derive(Debug, Default)]
#[must_use = "a builder does nothing until you call `.build()`"]
pub struct ReportBuilder {
    title: Option<String>,
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
    max_rows: Option<usize>,
}

impl ReportBuilder {
    /// An empty builder.
    pub fn new() -> Self {
        ReportBuilder::default()
    }

    /// Sets the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        todo!()
    }

    /// Adds a column.
    pub fn column(mut self, name: impl Into<String>) -> Self {
        todo!()
    }

    /// Adds a row.
    pub fn row<I, S>(mut self, cells: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        todo!()
    }

    /// Keeps at most `n` rows.
    pub fn max_rows(mut self, n: usize) -> Self {
        todo!()
    }

    /// Checks everything and builds the report.
    pub fn build(self) -> Result<Report, ReportError> {
        todo!()
    }
}

fn main() {
    let report = ReportBuilder::new()
        .title("Planets")
        .column("name")
        .column("moons")
        .row(["Earth", "1"])
        .row(vec![String::from("Mars"), String::from("2")])
        .build();
    match report {
        Ok(report) => print!("{}", report.render()),
        Err(error) => println!("error: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn planets() -> ReportBuilder {
        ReportBuilder::new()
            .title(String::from("Planets"))
            .column("name")
            .column("moons")
    }

    #[test]
    fn builds_a_report() {
        let report = planets()
            .row(["Earth", "1"])
            .row(vec![String::from("Mars"), String::from("2")])
            .row(["Jupiter", "95"].map(String::from))
            .build()
            .unwrap();
        assert_eq!(report.title(), "Planets");
        assert_eq!(report.rows().count(), 3);
        assert_eq!(
            report.render(),
            "Planets\nname | moons\nEarth | 1\nMars | 2\nJupiter | 95\n"
        );
    }

    #[test]
    fn max_rows_keeps_the_first_rows() {
        let report = planets()
            .row(["Earth", "1"])
            .row(["Mars", "2"])
            .max_rows(1)
            .build()
            .unwrap();
        let rows: Vec<&[String]> = report.rows().collect();
        assert_eq!(rows, [["Earth", "1"]]);
    }

    #[test]
    fn missing_parts_are_errors_in_order() {
        assert_eq!(ReportBuilder::new().build(), Err(ReportError::MissingTitle));
        assert_eq!(
            ReportBuilder::new().title("t").build(),
            Err(ReportError::NoColumns)
        );
        assert_eq!(planets().max_rows(0).build(), Err(ReportError::ZeroMaxRows));
    }

    #[test]
    fn rows_must_fit_the_columns() {
        let error = planets()
            .row(["Earth", "1"])
            .row(["Pluto"])
            .build()
            .unwrap_err();
        assert_eq!(
            error,
            ReportError::WrongWidth {
                row: 2,
                found: 1,
                expected: 2
            }
        );
        assert_eq!(
            error.to_string(),
            "row 2 has 1 cells, but there are 2 columns"
        );
    }
}
