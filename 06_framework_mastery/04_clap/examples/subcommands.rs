//! Subcommands, git style: `#[derive(Subcommand)]` on an enum, nested
//! subcommands, aliases, options shared with `#[command(flatten)]`, global
//! options, and an optional subcommand with a default action.
//! Run: cargo run -p clap_cli --example subcommands

use clap::{Args, Parser, Subcommand};

/// A pretend package manager
#[derive(Parser, Debug)]
#[command(name = "pkg")]
struct Pkg {
    /// Don't change anything, just say what would happen
    #[arg(long, global = true)] // allowed before OR after the subcommand
    dry_run: bool,

    #[command(subcommand)]
    command: Option<Cmd>, // Option: `pkg` alone is allowed
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Install packages (unit-like fields inline, in a struct variant)
    #[command(visible_alias = "i")]
    Install {
        /// Package names
        #[arg(required = true)]
        names: Vec<String>,
        #[command(flatten)]
        net: NetworkOptions,
    },
    /// Search for packages (a tuple variant wrapping an Args struct)
    Search(SearchArgs),
    /// Manage package sources (NESTED subcommands)
    #[command(subcommand)]
    Source(SourceCmd),
    /// Print the version (a variant with no arguments at all)
    Version,
}

/// Options several subcommands share.
#[derive(Args, Debug)]
struct NetworkOptions {
    /// Seconds to wait for the network
    #[arg(long, default_value_t = 30)]
    timeout: u32,
    /// Never touch the network
    #[arg(long)]
    offline: bool,
}

#[derive(Args, Debug)]
struct SearchArgs {
    /// What to look for
    query: String,
    #[command(flatten)]
    net: NetworkOptions,
}

#[derive(Subcommand, Debug)]
enum SourceCmd {
    /// Add a source
    Add { name: String, url: String },
    /// List sources
    List,
}

fn run(pkg: Pkg) -> String {
    let what = match pkg.command {
        None => "no subcommand: show a summary (the default action)".to_string(),
        Some(Cmd::Install { names, net }) => {
            format!(
                "install {names:?} (timeout {}s, offline {})",
                net.timeout, net.offline
            )
        }
        Some(Cmd::Search(args)) => format!("search for {:?}", args.query),
        Some(Cmd::Source(SourceCmd::Add { name, url })) => format!("add source {name} = {url}"),
        Some(Cmd::Source(SourceCmd::List)) => "list sources".to_string(),
        Some(Cmd::Version) => "pkg 1.0".to_string(),
    };
    if pkg.dry_run {
        format!("[dry run] {what}")
    } else {
        what
    }
}

fn show(args: &[&str]) {
    println!("$ {}", args.join(" "));
    match Pkg::try_parse_from(args) {
        Ok(pkg) => println!("  -> {}", run(pkg)),
        Err(e) => print!("{}", e.render()),
    }
}

fn main() {
    show(&["pkg"]);
    show(&["pkg", "install", "serde", "tokio", "--timeout", "5"]);
    show(&["pkg", "i", "clap", "--dry-run"]);
    show(&["pkg", "--dry-run", "search", "json", "--offline"]);
    show(&["pkg", "source", "add", "work", "https://example.com/pkgs"]);
    show(&["pkg", "source", "list"]);
    show(&["pkg", "source"]);
    show(&["pkg", "instal", "serde"]);
}
