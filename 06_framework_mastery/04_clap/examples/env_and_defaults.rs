//! Defaults and environment variables: a value can come from the command
//! line, from an environment variable, or from a default, in that order.
//! Run: cargo run -p clap_cli --example env_and_defaults
//!
//! Environment variables belong to a whole process, so to show different
//! ones this example runs ITSELF as a child process several times, setting
//! the variables on each child with `Command::env`. (Changing your own
//! process's environment with `std::env::set_var` is `unsafe` in Rust 2024,
//! because other threads might be reading it at the same moment.)

use clap::Parser;
use std::env;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(name = "server")]
struct Config {
    /// Port to listen on
    #[arg(long, env = "DEMO_PORT", default_value_t = 8080)]
    port: u16,

    /// Colour theme
    #[arg(long, env = "DEMO_THEME", default_value = "light")]
    theme: String,

    /// Turn on debug output (a flag can come from the environment too)
    #[arg(long, env = "DEMO_DEBUG")]
    debug: bool,
}

/// One run: the arguments, and the environment variables to set.
type Run = (
    &'static [&'static str],
    &'static [(&'static str, &'static str)],
);

fn main() {
    if env::var_os("DEMO_CHILD").is_some() {
        // The child: parse for real and report.
        let config = Config::parse();
        println!(
            "  -> port={} theme={} debug={}",
            config.port, config.theme, config.debug
        );
        return;
    }

    let me = env::current_exe().expect("own path");
    let runs: [Run; 5] = [
        (&[], &[]),
        (&[], &[("DEMO_PORT", "3000"), ("DEMO_THEME", "dark")]),
        (&["--port", "9000"], &[("DEMO_PORT", "3000")]),
        (&[], &[("DEMO_DEBUG", "true")]),
        (&[], &[("DEMO_PORT", "lots")]),
    ];
    for (args, vars) in runs {
        let mut shown: Vec<String> = vars.iter().map(|(k, v)| format!("{k}={v}")).collect();
        shown.push("server".to_string());
        shown.extend(args.iter().map(|a| a.to_string()));
        println!("$ {}", shown.join(" "));
        let output = Command::new(&me)
            .env("DEMO_CHILD", "1")
            .env_remove("DEMO_PORT") // start clean, whatever your shell has set
            .env_remove("DEMO_THEME")
            .env_remove("DEMO_DEBUG")
            .envs(vars.iter().copied())
            .args(args)
            .output()
            .expect("run myself");
        print!("{}", String::from_utf8_lossy(&output.stdout));
        print!("{}", String::from_utf8_lossy(&output.stderr));
        println!("  (exit code {})", output.status.code().unwrap_or(-1));
    }
}
