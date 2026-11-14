//! The command line, described as a struct. `#[derive(Parser)]` turns the
//! struct, its doc comments and its `#[arg]` attributes into a parser, the
//! `--help` text and the error messages. Lesson 06/04 covers clap in depth.

use std::path::PathBuf;

use capstone_cli_tool::{ColorWhen, Level, PartialSettings};
use clap::Parser;

/// Text printed after the options in `--help`.
const AFTER_HELP: &str = "\
Examples:
  logsum app.log                       summarise one file
  logsum app.log old.log --top 10      several files, ten top messages
  cat app.log | logsum                 read standard input
  logsum app.log --json                JSON for scripts (try: | jq .levels)
  logsum app.log --fail-on error       exit code 1 if there are errors (CI)

Settings, lowest priority first: built-in defaults, the config file
(logsum.toml), environment variables (LOGSUM_MIN_LEVEL, LOGSUM_TOP,
LOGSUM_COLOR), then flags. NO_COLOR=1 turns colours off.

Exit codes: 0 ok, 1 --fail-on level found, 2 error (usage, file or config).";

/// Summarise log files: count the lines at each level and list the most
/// frequent warnings and errors.
///
/// Understands lines such as "2026-05-01T09:00:00Z ERROR payment 991 failed"
/// and "[2026-05-01 09:00:05] [warn] disk 91% full". Numbers in messages are
/// replaced by # so that similar messages are counted together.
#[derive(Debug, Parser)]
#[command(name = "logsum", bin_name = "logsum", version, after_help = AFTER_HELP)]
pub struct Cli {
    /// Log files to read; - means standard input [default: -]
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    /// Lowest level to list under top messages: trace, debug, info, warn, error, fatal [default: warn]
    //
    // No `default_value` here: a default would make clap fill the field in
    // even when the user didn't type the flag, and then the config file and
    // environment could never apply. `None` means "not given".
    #[arg(short = 'l', long, value_name = "LEVEL")]
    pub min_level: Option<Level>,

    /// How many top messages to list [default: 5]
    #[arg(short = 'n', long, value_name = "N")]
    pub top: Option<usize>,

    /// When to use colours: auto, always, never [default: auto]
    #[arg(long, value_name = "WHEN")]
    pub color: Option<ColorWhen>,

    /// Print JSON instead of a table
    #[arg(long)]
    pub json: bool,

    /// Exit with code 1 if any line is at LEVEL or above
    #[arg(long, value_name = "LEVEL")]
    pub fail_on: Option<Level>,

    /// Read settings from this TOML file [env: LOGSUM_CONFIG] [default: ./logsum.toml if present]
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Print the settings and where each one came from, then exit
    #[arg(long)]
    pub show_config: bool,

    /// Never show the progress line
    #[arg(short, long)]
    pub quiet: bool,
}

impl Cli {
    /// The flags as the highest settings layer.
    pub fn settings_layer(&self) -> PartialSettings {
        PartialSettings {
            min_level: self.min_level,
            top: self.top,
            color: self.color,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use clap::error::ErrorKind;

    #[test]
    fn the_cli_definition_is_valid() {
        // Catches mistakes such as two flags with the same short letter,
        // which clap would otherwise only report when the program runs.
        Cli::command().debug_assert();
    }

    #[test]
    fn parses_files_and_flags_in_any_order() {
        let cli = Cli::try_parse_from([
            "logsum",
            "a.log",
            "-n",
            "3",
            "b.log",
            "--min-level",
            "ERROR",
            "--json",
        ])
        .unwrap();
        assert_eq!(cli.files, [PathBuf::from("a.log"), PathBuf::from("b.log")]);
        assert_eq!(cli.top, Some(3));
        assert_eq!(cli.min_level, Some(Level::Error));
        assert!(cli.json);
        assert_eq!(cli.color, None, "not given means None, not a default");
    }

    #[test]
    fn bad_values_are_usage_errors() {
        let err = Cli::try_parse_from(["logsum", "--top", "lots"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::ValueValidation);
        assert_eq!(err.exit_code(), 2);

        let err = Cli::try_parse_from(["logsum", "--min-level", "loud"]).unwrap_err();
        assert!(err.to_string().contains("unknown level \"loud\""), "{err}");

        let err = Cli::try_parse_from(["logsum", "--frobnicate"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::UnknownArgument);
    }

    #[test]
    fn help_and_version_are_errors_with_exit_code_0() {
        let err = Cli::try_parse_from(["logsum", "--help"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::DisplayHelp);
        assert_eq!(err.exit_code(), 0);
        let err = Cli::try_parse_from(["logsum", "-V"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::DisplayVersion);
    }

    #[test]
    fn flags_become_the_top_settings_layer() {
        let cli = Cli::try_parse_from(["logsum", "--color", "never"]).unwrap();
        assert_eq!(
            cli.settings_layer(),
            PartialSettings {
                color: Some(ColorWhen::Never),
                ..Default::default()
            }
        );
    }
}
