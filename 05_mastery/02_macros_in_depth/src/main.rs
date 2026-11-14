//! A settings checker for a pretend web server. `#[derive(Settings)]` writes
//! the loader and the help screen, `ensure!` checks the rules, and
//! `palette!` colours the output.
//!
//! Run:  cargo run -p macros_in_depth -- --help
//!       cargo run -p macros_in_depth -- name=demo port=9000
//!       cargo run -p macros_in_depth -- --file 05_mastery/02_macros_in_depth/server.conf
//!       cargo run -p macros_in_depth -- --file 05_mastery/02_macros_in_depth/server.conf port=80
//! Test: cargo test -p macros_in_depth

use std::io::IsTerminal;
use std::process::ExitCode;

use macros_in_depth::{Rgb, Settings, SettingsError, ensure, palette, parse_lines};

palette! {
    mod colours {
        heading = "#5fafff",
        good = "#5fd75f",
        bad = "#ff5f5f",
        dim = "#8a8a8a",
    }
}

/// How the server should start.
#[derive(Debug, Settings)]
struct ServerConfig {
    /// Name shown in every response.
    name: String,
    /// Host name or IP address to listen on.
    #[setting(default = "127.0.0.1")]
    host: String,
    /// Port to listen on.
    #[setting(default = "8080")]
    port: u16,
    /// Most connections served at the same time.
    #[setting(rename = "max-connections", default = "100")]
    max_connections: usize,
    /// Where to write the log; the terminal if not set.
    #[setting(rename = "log-file")]
    log_file: Option<String>,
}

/// The rules a type can't express on its own.
fn check(config: &ServerConfig) -> Result<(), SettingsError> {
    ensure!(
        config.port >= 1024,
        "port {} needs administrator rights: use 1024 or above",
        config.port
    );
    ensure!(
        (1..=10_000).contains(&config.max_connections),
        "max-connections must be between 1 and 10000, not {}",
        config.max_connections
    );
    Ok(())
}

/// Reads the optional `--file` and then `key=value` arguments, which win.
fn load(args: &[String]) -> Result<ServerConfig, String> {
    let mut map = std::collections::BTreeMap::new();
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        if arg == "--file" {
            let path = rest.next().ok_or("--file needs a path")?;
            let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
            map.extend(parse_lines(&text).map_err(|e| format!("{path}: {e}"))?);
        } else if let Some((key, value)) = arg.split_once('=') {
            map.insert(key.to_string(), value.to_string());
        } else {
            return Err(format!("expected key=value or --file PATH, found {arg:?}"));
        }
    }
    let config = ServerConfig::from_map_checked(&map).map_err(|e| e.to_string())?;
    check(&config).map_err(|e| e.to_string())?;
    Ok(config)
}

fn main() -> ExitCode {
    let colour = std::io::stdout().is_terminal();
    let paint = |rgb: Rgb, text: &str| {
        if colour {
            rgb.paint(text)
        } else {
            text.to_string()
        }
    };

    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--help") {
        println!(
            "{}",
            paint(colours::HEADING, "Server settings (key = value):")
        );
        print!("{}", ServerConfig::help());
        println!(
            "{}",
            paint(colours::DIM, "palette: ")
                + &colours::ALL
                    .iter()
                    .map(|(name, rgb)| paint(*rgb, &format!("{name} {rgb}")))
                    .collect::<Vec<_>>()
                    .join(", ")
        );
        return ExitCode::SUCCESS;
    }

    match load(&args) {
        Ok(config) => {
            println!("{}", paint(colours::GOOD, "settings OK"));
            println!(
                "  serving {:?} on {}:{}",
                config.name, config.host, config.port
            );
            println!("  up to {} connections at once", config.max_connections);
            match &config.log_file {
                Some(path) => println!("  logging to {path}"),
                None => println!("  logging to the terminal"),
            }
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{} {message}", paint(colours::BAD, "error:"));
            eprintln!("{}", paint(colours::DIM, "(--help lists every setting)"));
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn arguments_are_enough() {
        let config = load(&args(&["name=demo", "port=9000"])).unwrap();
        assert_eq!((config.name.as_str(), config.port), ("demo", 9000));
        assert_eq!(config.max_connections, 100);
    }

    #[test]
    fn the_sample_file_loads_and_arguments_override_it() {
        let file = concat!(env!("CARGO_MANIFEST_DIR"), "/server.conf");
        let config = load(&args(&["--file", file])).unwrap();
        assert_eq!(config.name, "Lesson server");
        let config = load(&args(&["--file", file, "port=7000"])).unwrap();
        assert_eq!(config.port, 7000);
    }

    #[test]
    fn rules_and_types_are_checked() {
        let low = load(&args(&["name=x", "port=80"])).unwrap_err();
        assert!(low.contains("administrator"), "{low}");
        let none = load(&args(&["name=x", "max-connections=0"])).unwrap_err();
        assert!(none.contains("between 1 and 10000"), "{none}");
        let typo = load(&args(&["name=x", "max_connections=5"])).unwrap_err();
        assert!(typo.contains("did you mean `max-connections`"), "{typo}");
    }
}
