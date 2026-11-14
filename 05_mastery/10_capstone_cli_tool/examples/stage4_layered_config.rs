//! Stage 4: settings from four layers. Built-in defaults < config file <
//! environment variables < flags. Each layer only overrides what it sets.
//! Run: cargo run -p capstone_cli_tool --example stage4_layered_config
//!
//! Everything here is in memory (a TOML string, a fake environment, fake
//! flags), so you can see the rules without creating files or touching the
//! real environment.

use std::collections::HashMap;
use std::path::Path;

use capstone_cli_tool::{ColorWhen, PartialSettings, Resolved, Source, resolve};

fn show(title: &str, resolved: &Resolved) {
    let Resolved { settings, sources } = resolved;
    println!("{title}");
    println!(
        "  min_level = {:<7} ({})",
        settings.min_level, sources.min_level
    );
    println!("  top       = {:<7} ({})", settings.top, sources.top);
    println!("  color     = {:<7} ({})", settings.color, sources.color);
}

fn main() -> Result<(), capstone_cli_tool::ConfigError> {
    // Layer 0: nothing at all. Settings::default() fills everything in.
    show("1. defaults only", &resolve(&[]));

    // Layer 1: the config file sets two things.
    let file =
        PartialSettings::from_toml("min_level = \"error\"\ntop = 3\n", Path::new("logsum.toml"))?;
    show(
        "2. + config file (min_level, top)",
        &resolve(&[(Source::File, file)]),
    );

    // Layer 2: the environment. A HashMap stands in for the real one; the
    // real program passes |key| std::env::var(key).ok().
    let fake_env: HashMap<&str, &str> = HashMap::from([("LOGSUM_TOP", "10")]);
    let env = PartialSettings::from_env(|key| fake_env.get(key).map(|v| v.to_string()))?;
    show(
        "3. + LOGSUM_TOP=10",
        &resolve(&[(Source::File, file), (Source::Env, env)]),
    );

    // Layer 3: flags, as clap would give them: None = "not typed".
    let flags = PartialSettings {
        color: Some(ColorWhen::Never),
        ..Default::default()
    };
    show(
        "4. + --color never",
        &resolve(&[
            (Source::File, file),
            (Source::Env, env),
            (Source::Flag, flags),
        ]),
    );

    // Typos in the file are errors, not silently ignored settings.
    let err = PartialSettings::from_toml("tpo = 3\n", Path::new("logsum.toml")).unwrap_err();
    println!("\na typo in the file: {err}");
    if let Some(cause) = std::error::Error::source(&err) {
        println!("{cause}");
    }
    Ok(())
}
