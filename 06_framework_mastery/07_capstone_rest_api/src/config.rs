//! Configuration: defaults, then an optional `KEY=VALUE` file, then the
//! environment. Later layers win, and everything is checked once at start-up.
//!
//! The loading is written as pure functions that take the file's text and a
//! "look up a variable" function, so tests can feed in any environment
//! without touching the real one (`std::env::set_var` is `unsafe` in Rust 2024).

use std::collections::HashMap;
use std::fmt;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

/// Every setting the server needs.
#[derive(Clone, PartialEq, Eq)]
pub struct Config {
    /// Address to listen on: `127.0.0.1` locally, `0.0.0.0` in a container.
    pub host: IpAddr,
    /// Port to listen on; `0` lets the OS pick a free one.
    pub port: u16,
    /// e.g. `sqlite:data/app.db?mode=rwc` or `sqlite::memory:`.
    pub database_url: String,
    /// Signs and checks access tokens. At least 32 bytes.
    pub jwt_secret: Secret,
    /// Who issues the tokens (the JWT `iss` claim).
    pub jwt_issuer: String,
    /// How long an access token lives.
    pub token_ttl: Duration,
    /// Argon2 cost: memory in KiB and passes. Lower only in tests.
    pub password_memory_kib: u32,
    /// Argon2 passes over the memory.
    pub password_iterations: u32,
    /// These e-mail addresses get the admin role when they register.
    pub admin_emails: Vec<String>,
    /// Longest a request may take before the server answers 408.
    pub request_timeout: Duration,
    /// `pretty` for people, `json` for log collectors.
    pub log_format: LogFormat,
}

/// How log lines look.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    /// Coloured, human-readable lines.
    Pretty,
    /// One JSON object per line.
    Json,
}

/// A string that never shows up in logs or `{:?}` output.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    /// Wraps a secret value.
    pub fn new(value: impl Into<String>) -> Secret {
        Secret(value.into())
    }

    /// The real value: call this only where it's needed.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(\"[redacted]\")")
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("database_url", &self.database_url)
            .field("jwt_secret", &self.jwt_secret)
            .field("jwt_issuer", &self.jwt_issuer)
            .field("token_ttl", &self.token_ttl)
            .field("password_memory_kib", &self.password_memory_kib)
            .field("password_iterations", &self.password_iterations)
            .field("admin_emails", &self.admin_emails)
            .field("request_timeout", &self.request_timeout)
            .field("log_format", &self.log_format)
            .finish()
    }
}

/// Why the configuration was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ConfigError {
    /// A required setting has no value anywhere.
    #[error("{0} is not set (see .env.example)")]
    Missing(&'static str),
    /// A setting has a value that doesn't make sense.
    #[error("{key} = {value:?}: {reason}")]
    Invalid {
        /// The variable.
        key: &'static str,
        /// What it was set to (secrets are not echoed).
        value: String,
        /// What's wrong with it.
        reason: String,
    },
    /// The config file couldn't be read or parsed.
    #[error("config file {path}: {reason}")]
    File {
        /// The file.
        path: String,
        /// What went wrong.
        reason: String,
    },
}

/// Smallest accepted JWT secret, in bytes (256 bits, as HS256 recommends).
pub const MIN_SECRET_LEN: usize = 32;

impl Config {
    /// Loads the real configuration: the file named by `APP_CONFIG_FILE` (if
    /// set), then the process environment on top.
    pub fn load() -> Result<Config, ConfigError> {
        let env = |key: &str| std::env::var(key).ok();
        let file = match env("APP_CONFIG_FILE") {
            Some(path) => {
                Some(
                    std::fs::read_to_string(&path).map_err(|err| ConfigError::File {
                        path: path.clone(),
                        reason: err.to_string(),
                    })?,
                )
            }
            None => None,
        };
        Config::from_sources(file.as_deref(), env)
    }

    /// Builds a config from a file's text (optional) and an environment
    /// lookup. Precedence: environment > file > default.
    pub fn from_sources(
        file: Option<&str>,
        env: impl Fn(&str) -> Option<String>,
    ) -> Result<Config, ConfigError> {
        let file_values = match file {
            Some(text) => parse_env_file(text)?,
            None => HashMap::new(),
        };
        let get = |key: &str| env(key).or_else(|| file_values.get(key).cloned());

        let host = parse_or(
            &get,
            "APP_HOST",
            "127.0.0.1".parse().expect("valid default"),
        )?;
        let port = parse_or(&get, "APP_PORT", 8080)?;
        let database_url = get("DATABASE_URL").ok_or(ConfigError::Missing("DATABASE_URL"))?;

        let secret = get("JWT_SECRET").ok_or(ConfigError::Missing("JWT_SECRET"))?;
        if secret.len() < MIN_SECRET_LEN {
            return Err(ConfigError::Invalid {
                key: "JWT_SECRET",
                value: "[redacted]".into(),
                reason: format!("must be at least {MIN_SECRET_LEN} bytes long"),
            });
        }

        let token_ttl = Duration::from_secs(parse_or(&get, "TOKEN_TTL_SECONDS", 3600)?);
        let password_memory_kib = parse_or(&get, "PASSWORD_MEMORY_KIB", 19 * 1024)?;
        let password_iterations = parse_or(&get, "PASSWORD_ITERATIONS", 2)?;
        let request_timeout = Duration::from_secs(parse_or(&get, "REQUEST_TIMEOUT_SECONDS", 10)?);
        let admin_emails = get("ADMIN_EMAILS")
            .map(|list| {
                list.split(',')
                    .map(|e| e.trim().to_lowercase())
                    .filter(|e| !e.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        let log_format = match get("LOG_FORMAT").as_deref() {
            None | Some("pretty") => LogFormat::Pretty,
            Some("json") => LogFormat::Json,
            Some(other) => {
                return Err(ConfigError::Invalid {
                    key: "LOG_FORMAT",
                    value: other.into(),
                    reason: "must be `pretty` or `json`".into(),
                });
            }
        };

        Ok(Config {
            host,
            port,
            database_url,
            jwt_secret: Secret(secret),
            jwt_issuer: get("JWT_ISSUER").unwrap_or_else(|| "capstone-rest-api".into()),
            token_ttl,
            password_memory_kib,
            password_iterations,
            admin_emails,
            request_timeout,
            log_format,
        })
    }

    /// A configuration for tests and examples: an in-memory database, a fixed
    /// (obviously fake) secret, and very cheap password hashing.
    pub fn for_tests() -> Config {
        Config {
            host: "127.0.0.1".parse().expect("valid address"),
            port: 0,
            database_url: "sqlite::memory:".into(),
            jwt_secret: Secret("test-secret-test-secret-test-secret!".into()),
            jwt_issuer: "capstone-rest-api".into(),
            token_ttl: Duration::from_secs(3600),
            password_memory_kib: 8, // the minimum: tests don't need slow hashing
            password_iterations: 1,
            admin_emails: vec!["admin@example.com".into()],
            request_timeout: Duration::from_secs(10),
            log_format: LogFormat::Pretty,
        }
    }

    /// The address to bind.
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }
}

/// Parses a `.env`-style file: `KEY=VALUE` lines, `#` comments, blank lines,
/// and optional quotes around the value.
pub fn parse_env_file(text: &str) -> Result<HashMap<String, String>, ConfigError> {
    let mut values = HashMap::new();
    for (number, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(ConfigError::File {
                path: "(text)".into(),
                reason: format!("line {}: expected KEY=VALUE", number + 1),
            });
        };
        let value = value.trim();
        let value = value
            .strip_prefix('"')
            .and_then(|v| v.strip_suffix('"'))
            .unwrap_or(value);
        values.insert(key.trim().to_string(), value.to_string());
    }
    Ok(values)
}

fn parse_or<T: std::str::FromStr>(
    get: &impl Fn(&str) -> Option<String>,
    key: &'static str,
    default: T,
) -> Result<T, ConfigError>
where
    T::Err: fmt::Display,
{
    match get(key) {
        None => Ok(default),
        Some(raw) => raw
            .trim()
            .parse()
            .map_err(|err: T::Err| ConfigError::Invalid {
                key,
                value: raw,
                reason: err.to_string(),
            }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "0123456789abcdef0123456789abcdef";

    fn env_of(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |key| map.get(key).cloned()
    }

    #[test]
    fn defaults_fill_in_everything_optional() {
        let env = env_of(&[("DATABASE_URL", "sqlite::memory:"), ("JWT_SECRET", SECRET)]);
        let config = Config::from_sources(None, env).unwrap();
        assert_eq!(config.socket_addr().to_string(), "127.0.0.1:8080");
        assert_eq!(config.token_ttl, Duration::from_secs(3600));
        assert_eq!(config.password_memory_kib, 19 * 1024);
        assert_eq!(config.log_format, LogFormat::Pretty);
    }

    #[test]
    fn the_environment_beats_the_file() {
        let file = "# a comment\nAPP_PORT=9000\nDATABASE_URL=\"sqlite:file.db\"\nJWT_SECRET=0123456789abcdef0123456789abcdef\nLOG_FORMAT=json\n";
        let env = env_of(&[
            ("APP_PORT", "7000"),
            ("ADMIN_EMAILS", "Ama@Example.com, kofi@example.com"),
        ]);
        let config = Config::from_sources(Some(file), env).unwrap();
        assert_eq!(config.port, 7000);
        assert_eq!(config.database_url, "sqlite:file.db");
        assert_eq!(config.log_format, LogFormat::Json);
        assert_eq!(config.admin_emails, ["ama@example.com", "kofi@example.com"]);
    }

    #[test]
    fn bad_values_are_refused_with_the_key_name() {
        let missing = Config::from_sources(None, env_of(&[("JWT_SECRET", SECRET)]));
        assert_eq!(missing.unwrap_err(), ConfigError::Missing("DATABASE_URL"));

        let short = Config::from_sources(
            None,
            env_of(&[("DATABASE_URL", "x"), ("JWT_SECRET", "short")]),
        );
        let message = short.unwrap_err().to_string();
        assert!(
            message.contains("JWT_SECRET") && !message.contains("short"),
            "{message}"
        );

        let port = Config::from_sources(
            None,
            env_of(&[
                ("DATABASE_URL", "x"),
                ("JWT_SECRET", SECRET),
                ("APP_PORT", "eighty"),
            ]),
        );
        assert_eq!(
            port.unwrap_err().to_string(),
            "APP_PORT = \"eighty\": invalid digit found in string"
        );
    }

    #[test]
    fn secrets_are_redacted_in_debug_output() {
        let shown = format!("{:?}", Config::for_tests());
        assert!(shown.contains("[redacted]"));
        assert!(!shown.contains("test-secret"));
    }

    #[test]
    fn the_example_file_is_a_valid_config() {
        let example = include_str!("../.env.example");
        let config = Config::from_sources(Some(example), |_| None).unwrap();
        assert_eq!(config.port, 8080);
        assert_eq!(config.admin_emails, ["admin@example.com"]);
    }

    #[test]
    fn a_malformed_file_line_is_an_error() {
        assert!(parse_env_file("JUST_A_WORD").is_err());
    }
}
