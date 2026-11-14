//! The command line, described with clap's derive API. Doc comments become
//! the help text; `#[command(..)]` configures a command, `#[arg(..)]` an
//! argument.

use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A tiny task tracker in the style of git: one tool, many subcommands.
#[derive(Debug, Parser)]
// `bin_name` fixes the name shown in "Usage:" (otherwise it is taken from
// how the program was started, e.g. `clap_cli.exe` on Windows).
#[command(name = "clap_cli", bin_name = "clap_cli", version, about)]
#[command(propagate_version = true)]
#[command(
    after_help = "Examples:\n  clap_cli add Buy milk -p high -t home\n  clap_cli list --tag home\n  clap_cli done 1"
)]
pub struct Cli {
    /// The JSON file the tasks are kept in
    #[arg(
        short,
        long,
        global = true,
        env = "TASKS_STORE",
        default_value = "tasks.json",
        value_name = "FILE"
    )]
    pub store: PathBuf,

    /// Say more about what happens (-v, -vv)
    #[arg(short, long, global = true, action = ArgAction::Count)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Add a task
    Add(AddArgs),
    /// List tasks
    #[command(visible_alias = "ls")]
    List(ListArgs),
    /// Mark tasks as done
    Done {
        /// Ids of the tasks to mark
        #[arg(required = true)]
        ids: Vec<u32>,
    },
    /// Remove tasks
    #[command(visible_alias = "rm")]
    Remove(RemoveArgs),
    /// Add or remove a task's tags
    #[command(subcommand)]
    Tag(TagCommand),
    /// Print a shell completion script
    Completions {
        /// The shell to write it for
        shell: Shell,
    },
}

#[derive(Debug, Args)]
pub struct AddArgs {
    /// What needs doing (several words are fine)
    #[arg(required = true)]
    pub title: Vec<String>,

    /// How urgent it is
    #[arg(short, long, value_enum, default_value_t = Priority::Normal)]
    pub priority: Priority,

    /// A tag; repeat for more: -t home -t errands
    #[arg(short, long = "tag", value_name = "TAG", value_parser = parse_tag)]
    pub tags: Vec<String>,

    /// Due date, as YYYY-MM-DD
    #[arg(short, long, value_name = "DATE", value_parser = parse_date)]
    pub due: Option<String>,
}

#[derive(Debug, Args)]
pub struct ListArgs {
    /// Which tasks to show
    #[arg(long, value_enum, default_value_t = Status::Open)]
    pub status: Status,

    /// Only tasks with this tag
    #[arg(short, long)]
    pub tag: Option<String>,

    /// Only tasks at least this urgent
    #[arg(short, long, value_enum)]
    pub priority: Option<Priority>,

    /// How to print them
    #[arg(short, long, value_enum, default_value_t = Format::Table)]
    pub format: Format,

    /// Show at most this many
    #[arg(short = 'n', long, value_parser = clap::value_parser!(u16).range(1..=1000))]
    pub limit: Option<u16>,
}

/// `remove` takes ids OR `--done`: exactly one of the two (an ArgGroup).
#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct RemoveArgs {
    /// Ids of the tasks to remove
    pub ids: Vec<u32>,

    /// Remove every finished task instead
    #[arg(long)]
    pub done: bool,
}

#[derive(Debug, Subcommand)]
pub enum TagCommand {
    /// Add tags to a task
    Add {
        /// The task's id
        id: u32,
        /// Tags to add
        #[arg(required = true, value_parser = parse_tag)]
        tags: Vec<String>,
    },
    /// Remove tags from a task
    #[command(visible_alias = "rm")]
    Remove {
        /// The task's id
        id: u32,
        /// Tags to remove
        #[arg(required = true)]
        tags: Vec<String>,
    },
}

/// `ValueEnum` turns each variant into an allowed value: `low`, `normal`,
/// `high`. Clap lists them in `--help` and in error messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Normal,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Status {
    Open,
    Done,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// Aligned columns, for people
    Table,
    /// One task per line: id and title, for scripts
    Plain,
    /// JSON, for other programs
    Json,
}

/// A custom value parser: any `fn(&str) -> Result<T, E>` works, and the
/// error text ends up in clap's message.
pub fn parse_tag(s: &str) -> Result<String, String> {
    let ok = !s.is_empty()
        && s.len() <= 20
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if ok {
        Ok(s.to_string())
    } else {
        Err("use 1 to 20 characters of a-z, 0-9 or -".to_string())
    }
}

/// Checks a `YYYY-MM-DD` date, including month lengths and leap years.
pub fn parse_date(s: &str) -> Result<String, String> {
    let bad = || format!("expected a date like 2026-12-31, got {s:?}");
    let parts: Vec<&str> = s.split('-').collect();
    let [y, m, d] = parts[..] else {
        return Err(bad());
    };
    if y.len() != 4 || m.len() != 2 || d.len() != 2 {
        return Err(bad());
    }
    let (year, month, day): (u32, u32, u32) = (
        y.parse().map_err(|_| bad())?,
        m.parse().map_err(|_| bad())?,
        d.parse().map_err(|_| bad())?,
    );
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return Err(format!("there is no month {month}")),
    };
    if day == 0 || day > days_in_month {
        return Err(format!("{year}-{m} has no day {day}"));
    }
    Ok(s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use clap::error::ErrorKind;

    #[test]
    fn the_cli_definition_is_valid() {
        // Catches mistakes clap can only find at run time: two args with
        // the same short flag, a default that isn't a possible value...
        Cli::command().debug_assert();
    }

    #[test]
    fn parses_add_with_defaults_and_repeated_tags() {
        let cli = Cli::try_parse_from([
            "clap_cli", "--store", "t.json", "add", "Buy", "milk", "-t", "home", "-t", "shop",
        ])
        .unwrap();
        assert_eq!(cli.store, PathBuf::from("t.json"));
        let Command::Add(add) = cli.command else {
            panic!("expected add")
        };
        assert_eq!(add.title, ["Buy", "milk"]);
        assert_eq!(add.priority, Priority::Normal);
        assert_eq!(add.tags, ["home", "shop"]);
        assert_eq!(add.due, None);
    }

    #[test]
    fn global_options_work_after_the_subcommand() {
        let cli = Cli::try_parse_from(["clap_cli", "ls", "-vv", "--store", "x.json"]).unwrap();
        assert_eq!(cli.verbose, 2);
        assert_eq!(cli.store, PathBuf::from("x.json"));
        assert!(matches!(cli.command, Command::List(_)));
    }

    #[test]
    fn bad_input_gives_the_right_error_kind() {
        let kind = |args: &[&str]| Cli::try_parse_from(args).unwrap_err().kind();
        assert_eq!(
            kind(&["clap_cli"]),
            ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
        );
        assert_eq!(
            kind(&["clap_cli", "frobnicate"]),
            ErrorKind::InvalidSubcommand
        );
        assert_eq!(
            kind(&["clap_cli", "add", "x", "-p", "urgent"]),
            ErrorKind::InvalidValue
        );
        assert_eq!(
            kind(&["clap_cli", "add", "x", "-d", "2026-02-30"]),
            ErrorKind::ValueValidation
        );
        assert_eq!(
            kind(&["clap_cli", "list", "-n", "0"]),
            ErrorKind::ValueValidation
        );
        assert_eq!(
            kind(&["clap_cli", "done"]),
            ErrorKind::MissingRequiredArgument
        );
        assert_eq!(
            kind(&["clap_cli", "rm", "1", "--done"]),
            ErrorKind::ArgumentConflict
        );
        assert_eq!(
            kind(&["clap_cli", "list", "--colour"]),
            ErrorKind::UnknownArgument
        );
        assert_eq!(kind(&["clap_cli", "--help"]), ErrorKind::DisplayHelp);
        assert_eq!(kind(&["clap_cli", "--version"]), ErrorKind::DisplayVersion);
    }

    #[test]
    fn dates_and_tags_are_validated() {
        assert!(parse_date("2028-02-29").is_ok()); // a leap year
        assert!(parse_date("2026-02-29").is_err());
        assert!(parse_date("2026-13-01").is_err());
        assert!(parse_date("26-1-1").is_err());
        assert!(parse_tag("errands-2").is_ok());
        assert!(parse_tag("Home").is_err());
        assert!(parse_tag("").is_err());
    }
}
