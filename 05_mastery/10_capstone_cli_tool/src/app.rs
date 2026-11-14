//! The application: glue between the outside world (arguments, environment,
//! files, terminal) and the library. Errors here are `anyhow::Error`: we only
//! need to *report* them, so one type that carries any error plus some
//! context is exactly right.

use std::env;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, IsTerminal, Write};
use std::path::Path;

use anyhow::{Context, Result};
use capstone_cli_tool::config::{ENV_COLOR, ENV_CONFIG, ENV_MIN_LEVEL, ENV_TOP};
use capstone_cli_tool::{
    Palette, PartialSettings, Progress, Resolved, Settings, Source, Summarizer, Summary,
    config_path, for_each_line, resolve, use_color, write_json, write_table,
};

use crate::cli::Cli;

/// How a successful run ended. `main` turns this into an exit code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// All done, nothing to report.
    Clean,
    /// `--fail-on` was given and a line at that level (or above) was found.
    ThresholdReached,
}

/// Runs logsum with parsed arguments.
pub fn run(cli: Cli) -> Result<Outcome> {
    let (resolved, config_file) = load_settings(&cli)?;
    let settings = resolved.settings;

    if cli.show_config {
        print_config(&resolved, config_file.as_deref())?;
        return Ok(Outcome::Clean);
    }

    // Colour goes to stdout, so ask whether *stdout* is a terminal.
    let colour = use_color(
        settings.color,
        io::stdout().is_terminal(),
        env::var("NO_COLOR").ok().as_deref(),
        env::var("TERM").ok().as_deref(),
    );
    // Progress goes to stderr, so ask about *stderr*.
    let show_progress = !cli.quiet && io::stderr().is_terminal();

    let summary = summarize_inputs(&cli.files, settings, show_progress)?;

    // Lock stdout once and buffer it: one system call instead of one per line.
    let mut out = BufWriter::new(io::stdout().lock());
    if cli.json {
        write_json(&mut out, &summary)?;
    } else {
        write_table(&mut out, &summary, Palette::new(colour))?;
    }
    out.flush()?; // a BufWriter dropped without flush() hides write errors

    let reached = cli
        .fail_on
        .is_some_and(|level| summary.levels.at_or_above(level) > 0);
    Ok(if reached {
        Outcome::ThresholdReached
    } else {
        Outcome::Clean
    })
}

/// Builds the four layers and resolves them. Also returns the config file
/// used, if any.
fn load_settings(cli: &Cli) -> Result<(Resolved, Option<String>)> {
    let current_dir = env::current_dir().context("cannot find the current folder")?;
    let mut layers = Vec::new();
    let mut used = None;

    if let Some(path) = config_path(
        cli.config.as_deref(),
        env::var(ENV_CONFIG).ok(),
        &current_dir,
    ) {
        // `?` converts the library's ConfigError into anyhow::Error. The
        // ConfigError keeps its own source (the io::Error or the TOML error),
        // so the whole chain is still there when main prints it.
        let layer = PartialSettings::from_file(&path)?;
        layers.push((Source::File, layer));
        used = Some(path.display().to_string());
    }

    layers.push((
        Source::Env,
        PartialSettings::from_env(|key| env::var(key).ok())?,
    ));
    layers.push((Source::Flag, cli.settings_layer()));
    Ok((resolve(&layers), used))
}

/// `--show-config`: every setting, its value and where it came from.
fn print_config(resolved: &Resolved, config_file: Option<&str>) -> Result<()> {
    let Resolved { settings, sources } = resolved;
    let env_name = |source: Source, var: &str| match source {
        Source::Env => format!("{source} ({var})"),
        _ => source.to_string(),
    };
    let mut out = io::stdout().lock();
    writeln!(out, "config file: {}", config_file.unwrap_or("(none)"))?;
    writeln!(
        out,
        "min_level = {:<8} from {}",
        settings.min_level,
        env_name(sources.min_level, ENV_MIN_LEVEL)
    )?;
    writeln!(
        out,
        "top       = {:<8} from {}",
        settings.top,
        env_name(sources.top, ENV_TOP)
    )?;
    writeln!(
        out,
        "color     = {:<8} from {}",
        settings.color,
        env_name(sources.color, ENV_COLOR)
    )?;
    Ok(())
}

/// Reads every input (or stdin) into one summary.
fn summarize_inputs(
    files: &[std::path::PathBuf],
    settings: Settings,
    progress: bool,
) -> Result<Summary> {
    let mut summarizer = Summarizer::new(settings.min_level);
    let stdin_only = [std::path::PathBuf::from("-")];
    let files = if files.is_empty() {
        &stdin_only[..]
    } else {
        files
    };

    for path in files {
        if path == Path::new("-") {
            let stdin = io::stdin();
            if stdin.is_terminal() {
                // Nobody piped anything in: say what we're waiting for.
                eprintln!(
                    "logsum: reading log lines from the keyboard; end with Ctrl+D (Ctrl+Z then Enter on Windows)"
                );
            }
            summarizer.start_input("-");
            read_one(
                stdin.lock(),
                "standard input",
                None,
                &mut summarizer,
                progress,
            )
            .context("cannot read standard input")?;
        } else {
            let file =
                File::open(path).with_context(|| format!("cannot open {}", path.display()))?;
            let size = file.metadata().ok().map(|m| m.len());
            summarizer.start_input(path.display().to_string());
            // 64 KiB buffer: fewer, bigger reads than the default 8 KiB.
            let reader = BufReader::with_capacity(64 * 1024, file);
            read_one(
                reader,
                &path.display().to_string(),
                size,
                &mut summarizer,
                progress,
            )
            .with_context(|| format!("cannot read {}", path.display()))?;
        }
    }
    Ok(summarizer.finish(settings.top))
}

/// Feeds one input to the summariser, drawing progress on stderr.
fn read_one(
    reader: impl io::BufRead,
    label: &str,
    size: Option<u64>,
    summarizer: &mut Summarizer,
    progress: bool,
) -> io::Result<()> {
    let mut bar = Progress::new(io::stderr(), progress, label, size);
    let bytes = for_each_line(reader, |line, so_far| {
        summarizer.add_line(line);
        bar.update(so_far);
    })?;
    bar.finish();
    summarizer.add_bytes(bytes);
    Ok(())
}
