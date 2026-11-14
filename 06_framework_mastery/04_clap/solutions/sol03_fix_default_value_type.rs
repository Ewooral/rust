//! Solution to exercise 3: make it compile (E0308: mismatched types).
//!
//! Clap has two ways to give an argument a default:
//!
//! - `default_value = "..."` is TEXT, parsed exactly as if the user had
//!   typed it (so it works for any type, and is shown as-is in `--help`);
//! - `default_value_t = expr` is a VALUE of the field's own type (shown in
//!   `--help` through `Display`).
//!
//! `host` is a `String`, but its `default_value_t` is a `&str`. Fix it
//! either way: switch to `default_value = "localhost"`, or keep
//! `default_value_t` and give it a `String`. Then check `-- --help` shows
//! both defaults.
//!
//! Run: cargo run -p clap_cli --features solutions --bin sol03_fix_default_value_type
//!      cargo run -p clap_cli --features solutions --bin sol03_fix_default_value_type -- --host example.com --port 8443

use clap::Parser;

/// Connects to a server (pretend)
#[derive(Parser, Debug)]
#[command(name = "connect")]
struct Cli {
    /// Server name
    // TEXT, parsed like user input. (Or: default_value_t = String::from("localhost").)
    #[arg(long, default_value = "localhost")]
    host: String,

    /// Server port
    #[arg(long, default_value_t = 8080)]
    port: u16,
}

fn main() {
    let cli = Cli::parse();
    println!("connecting to {}:{}", cli.host, cli.port);
}
