//! Stage 2: errors. `thiserror` for the LIBRARY (precise types callers can
//! match on), `anyhow` for the APPLICATION (one type that carries any error
//! plus context, for reporting).
//! Run: cargo run -p capstone_cli_tool --example stage2_errors

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::Context;
use thiserror::Error;

// ---------- the library side ----------

/// What can go wrong loading settings. A caller can `match` on this.
#[derive(Debug, Error)]
enum ConfigError {
    #[error("cannot read config file {}", .path.display())]
    Read {
        path: PathBuf,
        #[source] // the io::Error underneath: kept, not flattened into text
        source: io::Error,
    },
    #[error("`top` must be at least 1, not {0}")]
    TopIsZero(usize),
}

fn load_top(path: &Path) -> Result<usize, ConfigError> {
    let text = fs::read_to_string(path).map_err(|source| ConfigError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let top = text.trim().parse().unwrap_or(0);
    if top == 0 {
        return Err(ConfigError::TopIsZero(top));
    }
    Ok(top)
}

// ---------- the application side ----------

fn run() -> anyhow::Result<usize> {
    // `?` turns ConfigError into anyhow::Error; .context() adds a layer on top.
    let top = load_top(Path::new("no/such/logsum.conf")).context("could not load settings")?;
    Ok(top)
}

fn main() {
    let err = run().unwrap_err();

    println!("--- {{}}: only the outermost message");
    println!("{err}\n");

    println!("--- {{:#}}: the whole chain on one line");
    println!("{err:#}\n");

    println!("--- walking err.chain(), like logsum's main does");
    println!("error: {err}");
    for cause in err.chain().skip(1) {
        println!("  caused by: {cause}");
    }
    println!();

    println!("--- {{:?}}: what `fn main() -> anyhow::Result<()>` prints");
    println!("{err:?}\n");

    println!("--- the typed error is still inside, for code that cares");
    match err.downcast_ref::<ConfigError>() {
        Some(ConfigError::Read { path, .. }) => {
            println!("it was a Read error for {}", path.display())
        }
        Some(other) => println!("another ConfigError: {other}"),
        None => println!("not a ConfigError"),
    }
}
