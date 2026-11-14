//! Exercise 5: carry out subcommands.
//!
//! The command line of a tiny key-value tool is finished (the derive types
//! below). Implement `run`, which takes the PARSED `Cli` and an in-memory
//! store, and returns the text to print, or an error message:
//!
//! - `set KEY VALUE`: store it. Answer `set KEY = VALUE`, or, if the key
//!   already had a value, `updated KEY: OLD -> NEW`.
//! - `get KEY [--default D]`: the value; else the default if one was given;
//!   else `Err("no key KEY")`.
//! - `rm KEY...`: remove every key, all or nothing: if any key is missing,
//!   remove none and return `Err("no key KEY")` for the first missing one.
//!   Answer `removed N`.
//! - `list [--prefix P]`: `KEY=VALUE` lines in key order (a `BTreeMap` is
//!   already sorted), joined with `\n`, keeping only keys that start with
//!   the prefix; `(empty)` if there are none.
//!
//! Matching on the enum gives you each variant's fields by name:
//! `match cli.command { Cmd::Set { key, value } => ..., Cmd::Get { .. } => ... }`.
//!
//! Test: cargo test -p clap_cli --features exercises --bin ex05_subcommand_dispatch

#![allow(unused_variables, dead_code)]

use clap::{Parser, Subcommand};
use std::collections::BTreeMap;

/// A tiny key-value store
#[derive(Parser, Debug)]
#[command(name = "kv")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Store a value
    Set { key: String, value: String },
    /// Read a value
    Get {
        key: String,
        /// What to answer if the key is missing
        #[arg(long)]
        default: Option<String>,
    },
    /// Remove keys
    Rm {
        #[arg(required = true)]
        keys: Vec<String>,
    },
    /// List keys and values
    List {
        /// Only keys starting with this
        #[arg(long)]
        prefix: Option<String>,
    },
}

fn run(cli: Cli, store: &mut BTreeMap<String, String>) -> Result<String, String> {
    todo!()
}

fn main() {
    let mut store = BTreeMap::new();
    for line in [
        "kv set colour red",
        "kv set size big",
        "kv set colour blue",
        "kv list",
        "kv get shape --default round",
        "kv rm size",
    ] {
        let cli = Cli::try_parse_from(line.split(' ')).unwrap();
        println!("$ {line}\n{:?}", run(cli, &mut store));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kv(store: &mut BTreeMap<String, String>, line: &str) -> Result<String, String> {
        let cli = Cli::try_parse_from(line.split_whitespace()).expect("valid command line");
        run(cli, store)
    }

    #[test]
    fn set_and_get() {
        let mut store = BTreeMap::new();
        assert_eq!(
            kv(&mut store, "kv set colour red"),
            Ok("set colour = red".into())
        );
        assert_eq!(
            kv(&mut store, "kv set colour blue"),
            Ok("updated colour: red -> blue".into())
        );
        assert_eq!(kv(&mut store, "kv get colour"), Ok("blue".into()));
        assert_eq!(kv(&mut store, "kv get shape"), Err("no key shape".into()));
        assert_eq!(
            kv(&mut store, "kv get shape --default round"),
            Ok("round".into())
        );
    }

    #[test]
    fn rm_is_all_or_nothing() {
        let mut store = BTreeMap::new();
        kv(&mut store, "kv set a 1").unwrap();
        kv(&mut store, "kv set b 2").unwrap();
        assert_eq!(kv(&mut store, "kv rm a zzz"), Err("no key zzz".into()));
        assert_eq!(store.len(), 2, "nothing was removed");
        assert_eq!(kv(&mut store, "kv rm a b"), Ok("removed 2".into()));
        assert!(store.is_empty());
    }

    #[test]
    fn list_sorted_and_filtered() {
        let mut store = BTreeMap::new();
        assert_eq!(kv(&mut store, "kv list"), Ok("(empty)".into()));
        for line in [
            "kv set web.port 80",
            "kv set db.host x",
            "kv set web.host y",
        ] {
            kv(&mut store, line).unwrap();
        }
        assert_eq!(
            kv(&mut store, "kv list"),
            Ok("db.host=x\nweb.host=y\nweb.port=80".into())
        );
        assert_eq!(
            kv(&mut store, "kv list --prefix web."),
            Ok("web.host=y\nweb.port=80".into())
        );
    }
}
