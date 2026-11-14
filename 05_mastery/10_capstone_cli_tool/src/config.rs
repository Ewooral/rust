//! Settings from four layers: built-in defaults, a config file, environment
//! variables and command-line flags. Each layer overrides the ones before it.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::level::Level;
use crate::term::ColorWhen;

/// The file logsum looks for in the current folder when no `--config` or
/// `LOGSUM_CONFIG` is given.
pub const DEFAULT_CONFIG_FILE: &str = "logsum.toml";

/// The environment variables logsum reads. `NO_COLOR` is handled by
/// [`crate::term::use_color`], because it isn't a logsum setting.
pub const ENV_CONFIG: &str = "LOGSUM_CONFIG";
pub const ENV_MIN_LEVEL: &str = "LOGSUM_MIN_LEVEL";
pub const ENV_TOP: &str = "LOGSUM_TOP";
pub const ENV_COLOR: &str = "LOGSUM_COLOR";

/// The final, complete settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Settings {
    /// The lowest level listed under "top messages".
    pub min_level: Level,
    /// How many top messages to list.
    pub top: usize,
    /// When to colour the output.
    pub color: ColorWhen,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            min_level: Level::Warn,
            top: 5,
            color: ColorWhen::Auto,
        }
    }
}

/// One layer: any setting may be missing (`None`) and is then left alone.
///
/// This is also the shape of the config file. `deny_unknown_fields` turns a
/// typo such as `min_levle = "warn"` into an error instead of silently
/// ignoring it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialSettings {
    pub min_level: Option<Level>,
    pub top: Option<usize>,
    pub color: Option<ColorWhen>,
}

/// Where a setting's final value came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Default,
    File,
    Env,
    Flag,
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(match self {
            Source::Default => "default",
            Source::File => "config file",
            Source::Env => "environment",
            Source::Flag => "flag",
        })
    }
}

/// Which layer each setting came from (for `--show-config`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Sources {
    pub min_level: Source,
    pub top: Source,
    pub color: Source,
}

/// The settings plus where each one came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resolved {
    pub settings: Settings,
    pub sources: Sources,
}

/// Everything that can go wrong while reading settings.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// The file exists (or was asked for) but couldn't be read.
    #[error("cannot read config file {}", .path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    /// The file isn't valid TOML, or has unknown keys or bad values.
    #[error("invalid config file {}", .path.display())]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    /// An environment variable has a value we can't use.
    #[error("invalid value {value:?} in environment variable {var}: {reason}")]
    Env {
        var: &'static str,
        value: String,
        reason: String,
    },
}

impl PartialSettings {
    /// Parses the text of a config file. `path` is only used in errors.
    ///
    /// ```
    /// use capstone_cli_tool::{Level, PartialSettings};
    /// let layer = PartialSettings::from_toml("min_level = \"error\"", "logsum.toml".as_ref())?;
    /// assert_eq!(layer.min_level, Some(Level::Error));
    /// assert_eq!(layer.top, None);
    /// # Ok::<(), capstone_cli_tool::ConfigError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// [`ConfigError::Parse`] if the text isn't valid TOML, has an unknown
    /// key, or has a value of the wrong type.
    pub fn from_toml(text: &str, path: &Path) -> Result<Self, ConfigError> {
        toml::from_str(text).map_err(|source| ConfigError::Parse {
            path: path.to_path_buf(),
            source,
        })
    }

    /// Reads and parses a config file.
    ///
    /// # Errors
    ///
    /// [`ConfigError::Read`] if the file can't be read, otherwise as
    /// [`PartialSettings::from_toml`].
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let text = fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_toml(&text, path)
    }

    /// Reads the `LOGSUM_*` variables through `get`, so tests can pass a
    /// closure over a map instead of changing the real environment (which
    /// is `unsafe` in the 2024 edition). The app passes
    /// `|key| std::env::var(key).ok()`.
    ///
    /// Empty values count as unset.
    ///
    /// # Errors
    ///
    /// [`ConfigError::Env`] if a variable is set to a value that can't be
    /// parsed.
    pub fn from_env(get: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        fn read<T: std::str::FromStr>(
            get: &impl Fn(&str) -> Option<String>,
            var: &'static str,
        ) -> Result<Option<T>, ConfigError>
        where
            T::Err: fmt::Display,
        {
            match get(var).filter(|value| !value.is_empty()) {
                None => Ok(None),
                Some(value) => match value.parse() {
                    Ok(parsed) => Ok(Some(parsed)),
                    Err(e) => Err(ConfigError::Env {
                        var,
                        reason: e.to_string(),
                        value,
                    }),
                },
            }
        }

        Ok(PartialSettings {
            min_level: read(&get, ENV_MIN_LEVEL)?,
            top: read(&get, ENV_TOP)?,
            color: read(&get, ENV_COLOR)?,
        })
    }
}

/// Applies the layers in order, each overriding the ones before it, on top
/// of [`Settings::default`].
///
/// ```
/// use capstone_cli_tool::{PartialSettings, Source, resolve};
/// let file = PartialSettings { top: Some(10), ..Default::default() };
/// let flags = PartialSettings { top: Some(3), ..Default::default() };
/// let resolved = resolve(&[(Source::File, file), (Source::Flag, flags)]);
/// assert_eq!(resolved.settings.top, 3);
/// assert_eq!(resolved.sources.top, Source::Flag);
/// ```
pub fn resolve(layers: &[(Source, PartialSettings)]) -> Resolved {
    /// If this layer has a value, it wins: store it and remember where it came from.
    fn apply<T: Copy>(slot: &mut T, origin: &mut Source, value: Option<T>, source: Source) {
        if let Some(value) = value {
            *slot = value;
            *origin = source;
        }
    }

    let mut settings = Settings::default();
    let mut sources = Sources {
        min_level: Source::Default,
        top: Source::Default,
        color: Source::Default,
    };
    for &(source, layer) in layers {
        apply(
            &mut settings.min_level,
            &mut sources.min_level,
            layer.min_level,
            source,
        );
        apply(&mut settings.top, &mut sources.top, layer.top, source);
        apply(&mut settings.color, &mut sources.color, layer.color, source);
    }
    Resolved { settings, sources }
}

/// Which config file to read, if any: `--config`, then `LOGSUM_CONFIG`,
/// then `logsum.toml` in `current_dir` if it exists.
///
/// A file the user named is returned even if it doesn't exist, so that
/// reading it fails loudly. The default file is only used if it's there.
pub fn config_path(
    flag: Option<&Path>,
    env: Option<String>,
    current_dir: &Path,
) -> Option<PathBuf> {
    if let Some(path) = flag {
        return Some(path.to_path_buf());
    }
    if let Some(path) = env.filter(|p| !p.is_empty()) {
        return Some(PathBuf::from(path));
    }
    let default = current_dir.join(DEFAULT_CONFIG_FILE);
    default.is_file().then_some(default)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::error::Error;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |key| map.get(key).cloned()
    }

    #[test]
    fn defaults_when_no_layer_says_anything() {
        let resolved = resolve(&[]);
        assert_eq!(resolved.settings, Settings::default());
        assert_eq!(resolved.sources.top, Source::Default);
    }

    #[test]
    fn later_layers_win_setting_by_setting() {
        let file = PartialSettings {
            min_level: Some(Level::Info),
            top: Some(10),
            color: Some(ColorWhen::Never),
        };
        let env_layer = PartialSettings {
            top: Some(20),
            ..Default::default()
        };
        let flags = PartialSettings {
            color: Some(ColorWhen::Always),
            ..Default::default()
        };
        let r = resolve(&[
            (Source::File, file),
            (Source::Env, env_layer),
            (Source::Flag, flags),
        ]);
        assert_eq!(
            r.settings,
            Settings {
                min_level: Level::Info,
                top: 20,
                color: ColorWhen::Always
            }
        );
        assert_eq!(
            r.sources,
            Sources {
                min_level: Source::File,
                top: Source::Env,
                color: Source::Flag
            }
        );
    }

    #[test]
    fn toml_layer_parses_and_rejects_typos() {
        let path = Path::new("logsum.toml");
        let layer = PartialSettings::from_toml("top = 7\ncolor = \"never\"\n", path).unwrap();
        assert_eq!(layer.top, Some(7));
        assert_eq!(layer.color, Some(ColorWhen::Never));

        let err = PartialSettings::from_toml("min_levle = \"warn\"\n", path).unwrap_err();
        assert_eq!(err.to_string(), "invalid config file logsum.toml");
        let cause = err
            .source()
            .expect("the TOML error is the source")
            .to_string();
        assert!(cause.contains("unknown field `min_levle`"), "{cause}");
    }

    #[test]
    fn env_layer_reads_parses_and_reports_bad_values() {
        let layer = PartialSettings::from_env(env(&[
            ("LOGSUM_TOP", "3"),
            ("LOGSUM_MIN_LEVEL", "ERROR"),
            ("LOGSUM_COLOR", ""), // empty counts as unset
        ]))
        .unwrap();
        assert_eq!(layer.top, Some(3));
        assert_eq!(layer.min_level, Some(Level::Error));
        assert_eq!(layer.color, None);

        let err = PartialSettings::from_env(env(&[("LOGSUM_TOP", "many")])).unwrap_err();
        assert_eq!(
            err.to_string(),
            "invalid value \"many\" in environment variable LOGSUM_TOP: invalid digit found in string"
        );
    }

    #[test]
    fn missing_file_is_a_read_error_with_the_io_error_as_source() {
        let err = PartialSettings::from_file(Path::new("definitely/not/here.toml")).unwrap_err();
        assert!(matches!(err, ConfigError::Read { .. }));
        let io = err.source().unwrap().downcast_ref::<io::Error>().unwrap();
        assert_eq!(io.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn config_path_prefers_flag_then_env_then_default_file() {
        let here = Path::new("no/such/folder");
        assert_eq!(
            config_path(Some(Path::new("a.toml")), Some("b.toml".into()), here),
            Some(PathBuf::from("a.toml"))
        );
        assert_eq!(
            config_path(None, Some("b.toml".into()), here),
            Some(PathBuf::from("b.toml"))
        );
        assert_eq!(config_path(None, Some(String::new()), here), None);
        assert_eq!(config_path(None, None, here), None);
    }
}
