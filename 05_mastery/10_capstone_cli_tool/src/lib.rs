//! # logsum: summarise log files
//!
//! The library half of the capstone CLI tool. It knows nothing about
//! command-line arguments, exit codes or the real terminal: it parses log
//! lines, counts them, layers settings and renders output, and every error it
//! returns is a precise `thiserror` type. The application half
//! (`src/main.rs`, `src/cli.rs`, `src/app.rs`) wires it to the outside world
//! with `clap` and `anyhow`.
//!
//! ```
//! use capstone_cli_tool::{Level, summarize_text};
//!
//! let log = "\
//! 2026-05-01T09:00:00Z INFO server started
//! 2026-05-01T09:00:02Z ERROR payment 991 failed
//! 2026-05-01T09:00:03Z ERROR payment 17 failed
//! ";
//! let summary = summarize_text(log, Level::Warn, 5);
//! assert_eq!(summary.levels.error, 2);
//! assert_eq!(summary.top[0].pattern, "payment # failed");
//! assert_eq!(summary.top[0].count, 2);
//! ```

pub mod config;
pub mod input;
pub mod level;
pub mod record;
pub mod render;
pub mod summary;
pub mod term;

pub use config::{
    ConfigError, PartialSettings, Resolved, Settings, Source, Sources, config_path, resolve,
};
pub use input::for_each_line;
pub use level::{Level, ParseLevelError};
pub use record::{Record, normalize, parse_line};
pub use render::{group_digits, write_json, write_table};
pub use summary::{LevelCounts, Summarizer, Summary, TopMessage, summarize_text};
pub use term::{ColorWhen, Palette, ParseColorError, Progress, human_bytes, use_color};
