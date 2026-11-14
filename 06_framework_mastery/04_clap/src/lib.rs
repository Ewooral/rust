//! Mini project: `clap_cli`, a git-style task tracker over a JSON file.
//!
//! - [`cli`]: the whole command line as clap derive types: global options
//!   (`--store` with an env var and a default, `-v` counted), subcommands
//!   (one with nested subcommands), `ValueEnum`s, custom value parsers, an
//!   argument group;
//! - [`store`]: the tasks, the JSON file and `AppError`;
//! - [`run`]: carries out a parsed `Cli`, writing to any `Write`, so tests
//!   can capture the output without running a process.
//!
//! `src/main.rs` is only: parse, run, choose the exit code. `tests/cli.rs`
//! runs the real binary.

pub mod cli;
pub mod store;

pub use cli::{Cli, Command, Format, Priority, Status, TagCommand};
pub use store::{AppError, Query, Store, Task};

use clap::CommandFactory;
use std::io::{self, Write};

/// Carries out the command. Normal output goes to `out`; `-v` notes go to
/// `err`, so they never mix with output a script might be reading.
pub fn run(cli: Cli, out: &mut dyn Write, err: &mut dyn Write) -> Result<(), AppError> {
    if let Command::Completions { shell } = cli.command {
        // Completions don't touch the task file at all.
        clap_complete::generate(shell, &mut Cli::command(), "clap_cli", out);
        return Ok(());
    }

    let path = cli.store.as_path();
    let mut store = Store::load(path)?;
    if cli.verbose > 0 {
        note(
            err,
            &format!("using {} ({} tasks)", path.display(), store.tasks().len()),
        );
    }

    let changed = match cli.command {
        Command::Add(add) => {
            let task = store.add(add.title.join(" "), add.priority, add.tags, add.due);
            say(out, &format!("added task {}: {}", task.id, task.title));
            true
        }
        Command::List(list) => {
            let query = Query {
                status: Some(list.status),
                tag: list.tag,
                min_priority: list.priority,
                limit: list.limit.map(usize::from),
            };
            print_tasks(out, &store.query(&query), list.format);
            false
        }
        Command::Done { ids } => {
            store.mark_done(&ids)?;
            let list: Vec<String> = ids.iter().map(u32::to_string).collect();
            say(out, &format!("done: {}", list.join(", ")));
            true
        }
        Command::Remove(remove) => {
            let removed = if remove.done {
                store.remove_done()
            } else {
                store.remove(&remove.ids)?
            };
            say(out, &format!("removed {removed} task(s)"));
            removed > 0
        }
        Command::Tag(TagCommand::Add { id, tags }) => {
            let task = store.add_tags(id, &tags)?;
            say(out, &format!("task {id} tags: {}", task.tags.join(", ")));
            true
        }
        Command::Tag(TagCommand::Remove { id, tags }) => {
            let task = store.remove_tags(id, &tags)?;
            say(out, &format!("task {id} tags: {}", task.tags.join(", ")));
            true
        }
        Command::Completions { .. } => unreachable!("handled above"),
    };

    if changed {
        store.save(path)?;
        if cli.verbose > 1 {
            note(err, &format!("saved {}", path.display()));
        }
    }
    Ok(())
}

fn print_tasks(out: &mut dyn Write, tasks: &[&Task], format: Format) {
    match format {
        Format::Json => {
            let json = serde_json::to_string_pretty(tasks).expect("tasks always serialize");
            say(out, &json);
        }
        Format::Plain => {
            for task in tasks {
                say(out, &format!("{}\t{}", task.id, task.title));
            }
        }
        Format::Table if tasks.is_empty() => say(out, "no tasks"),
        Format::Table => {
            say(
                out,
                &format!(
                    "{:>3}  {:<6}  {:<4}  {:<10}  {:<14}  TITLE",
                    "ID", "PRI", "DONE", "DUE", "TAGS"
                ),
            );
            for t in tasks {
                let priority = format!("{:?}", t.priority).to_lowercase();
                say(
                    out,
                    &format!(
                        "{:>3}  {:<6}  {:<4}  {:<10}  {:<14}  {}",
                        t.id,
                        priority,
                        if t.done { "yes" } else { "" },
                        t.due.as_deref().unwrap_or("-"),
                        t.tags.join(","),
                        t.title
                    ),
                );
            }
        }
    }
}

/// Writes one line of output. A closed pipe (`clap_cli list | head -1`)
/// is not worth crashing over, so write errors are ignored.
fn say(out: &mut dyn Write, line: &str) {
    let _ = writeln!(out, "{line}");
}

fn note(err: &mut dyn Write, line: &str) {
    let _ = writeln!(err, "note: {line}");
}

/// Runs the tool on `args` in-process and returns (stdout, result). Used by
/// the unit tests below and handy in the examples.
pub fn run_args(args: &[&str]) -> Result<(String, Result<(), AppError>), clap::Error> {
    use clap::Parser;
    let cli = Cli::try_parse_from(args)?;
    let mut out = Vec::new();
    let result = run(cli, &mut out, &mut io::sink());
    Ok((String::from_utf8(out).expect("UTF-8 output"), result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_list_done_in_process() {
        let dir = tempfile::tempdir().unwrap();
        let store = dir.path().join("t.json");
        let store = store.to_str().unwrap();
        let run = |args: &[&str]| {
            let mut full = vec!["clap_cli", "--store", store];
            full.extend_from_slice(args);
            let (out, result) = run_args(&full).unwrap();
            result.unwrap();
            out
        };
        assert_eq!(
            run(&["add", "Buy", "milk", "-t", "home"]),
            "added task 1: Buy milk\n"
        );
        assert_eq!(
            run(&["add", "Call", "Ada", "-p", "high"]),
            "added task 2: Call Ada\n"
        );
        assert_eq!(run(&["list", "-f", "plain"]), "1\tBuy milk\n2\tCall Ada\n");
        assert_eq!(run(&["done", "1"]), "done: 1\n");
        assert_eq!(run(&["list", "-f", "plain"]), "2\tCall Ada\n");
        assert_eq!(
            run(&["list", "--status", "all", "-f", "plain", "-t", "home"]),
            "1\tBuy milk\n"
        );
    }

    #[test]
    fn unknown_ids_are_app_errors() {
        let dir = tempfile::tempdir().unwrap();
        let store = dir.path().join("t.json");
        let (_, result) =
            run_args(&["clap_cli", "-s", store.to_str().unwrap(), "done", "7"]).unwrap();
        assert_eq!(result.unwrap_err().to_string(), "no task with id 7");
    }

    #[test]
    fn completions_need_no_task_file() {
        let (out, result) = run_args(&["clap_cli", "completions", "bash"]).unwrap();
        result.unwrap();
        assert!(
            out.contains("_clap_cli()"),
            "{}",
            &out[..200.min(out.len())]
        );
    }
}
