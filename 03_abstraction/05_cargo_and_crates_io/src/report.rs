//! Turning findings into text (or JSON, with the `json` feature).

use crate::checks::{Finding, Severity, passes};

// Conditional compilation (Beyond the Book): exactly ONE of these two
// constants exists in any build, chosen by the `unicode` feature.
// `cargo run -p cargo_and_crates_io --no-default-features` picks the second.

/// The marks used for each severity, then for "passed".
#[cfg(feature = "unicode")]
pub const MARKS: [&str; 4] = ["✗", "⚠", "ℹ", "✓"];

/// The marks used for each severity, then for "passed".
#[cfg(not(feature = "unicode"))]
pub const MARKS: [&str; 4] = ["[x]", "[!]", "[i]", "[ok]"];

/// The mark for one severity.
pub fn mark(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => MARKS[0],
        Severity::Warning => MARKS[1],
        Severity::Info => MARKS[2],
    }
}

/// Counts errors and warnings: `(errors, warnings)`.
pub fn counts(findings: &[Finding]) -> (usize, usize) {
    let errors = findings
        .iter()
        .filter(|f| f.severity == Severity::Error)
        .count();
    let warnings = findings
        .iter()
        .filter(|f| f.severity == Severity::Warning)
        .count();
    (errors, warnings)
}

/// A full report for one crate, one finding per line.
///
/// # Examples
///
/// ```
/// use cargo_and_crates_io::report::render;
/// use cargo_and_crates_io::{Finding, Severity};
///
/// let findings = [Finding::new(Severity::Warning, "repository", "no link")];
/// let text = render("paint 0.1.0", &findings);
/// assert!(text.contains("repository"));
/// assert!(text.ends_with("Ready for crates.io: yes (0 errors, 1 warning)\n"));
/// ```
pub fn render(title: &str, findings: &[Finding]) -> String {
    let mut out = format!("{title}\n");
    for f in findings {
        out.push_str(&format!(
            "  {} {:<12} {}\n",
            mark(f.severity),
            f.field,
            f.message
        ));
    }
    let (errors, warnings) = counts(findings);
    let verdict = if passes(findings) { "yes" } else { "no" };
    out.push_str(&format!(
        "Ready for crates.io: {verdict} ({errors} error{}, {warnings} warning{})\n",
        plural(errors),
        plural(warnings)
    ));
    out
}

/// One line for a crate in the workspace summary.
pub fn summary_line(name: &str, findings: &[Finding]) -> String {
    let (errors, warnings) = counts(findings);
    let first_mark = if passes(findings) { MARKS[3] } else { MARKS[0] };
    let problems: Vec<&str> = findings
        .iter()
        .filter(|f| f.severity == Severity::Error)
        .map(|f| f.field)
        .collect();
    let detail = if problems.is_empty() {
        String::new()
    } else {
        format!("  missing or wrong: {}", problems.join(", "))
    };
    format!("  {first_mark:<4} {name:<28} {errors} err  {warnings} warn{detail}")
}

/// The report as JSON. Only exists with `--features json`.
///
/// # Errors
///
/// Returns the `serde_json` error if the findings cannot be serialised
/// (with these plain types that never happens in practice).
#[cfg(feature = "json")]
pub fn render_json(name: &str, findings: &[Finding]) -> Result<String, serde_json::Error> {
    #[derive(serde::Serialize)]
    struct Report<'a> {
        name: &'a str,
        ready: bool,
        findings: &'a [Finding],
    }
    serde_json::to_string_pretty(&Report {
        name,
        ready: passes(findings),
        findings,
    })
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_lists_each_finding_and_a_verdict() {
        let findings = [
            Finding::new(Severity::Error, "license", "missing"),
            Finding::new(Severity::Info, "publish", "off"),
        ];
        let text = render("paint 0.1.0", &findings);
        assert_eq!(text.lines().count(), 4);
        assert!(text.contains(mark(Severity::Error)));
        assert!(text.ends_with("Ready for crates.io: no (1 error, 0 warnings)\n"));
    }

    #[test]
    fn summary_names_the_failing_fields() {
        let findings = [
            Finding::new(Severity::Error, "description", "missing"),
            Finding::new(Severity::Error, "license", "missing"),
        ];
        let line = summary_line("paint", &findings);
        assert!(
            line.ends_with("missing or wrong: description, license"),
            "{line}"
        );
        assert!(summary_line("paint", &[]).starts_with(&format!("  {}", MARKS[3])));
    }

    #[cfg(feature = "json")]
    #[test]
    fn json_output_is_machine_readable() {
        let findings = [Finding::new(Severity::Warning, "repository", "no link")];
        let json = render_json("paint", &findings).unwrap();
        assert!(json.contains("\"severity\": \"warning\""), "{json}");
        assert!(json.contains("\"ready\": true"));
    }
}
