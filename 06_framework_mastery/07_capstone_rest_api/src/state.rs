//! The application state every handler can reach with `State<AppState>`,
//! and how it is built at start-up: pool, migrations, then the services.

use std::str::FromStr;
use std::sync::Arc;

use sqlx::SqlitePool;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use crate::auth::{Passwords, Tokens};
use crate::config::Config;
use crate::error::AppError;
use crate::repository::{ReviewRepository, UserRepository};
use crate::service::{AuthService, ReviewService};

/// The migrations in `migrations/`, embedded at compile time.
pub static MIGRATOR: Migrator = sqlx::migrate!();

/// Shared by every request. Cloning is cheap: everything inside is a pool
/// handle, an `Arc`, or small.
#[derive(Clone)]
pub struct AppState {
    /// The settings the server started with.
    pub config: Arc<Config>,
    /// The connection pool (handlers use it only for `/health`).
    pub db: SqlitePool,
    /// Accounts and credentials.
    pub auth: AuthService,
    /// Reviews.
    pub reviews: ReviewService,
}

/// Why the server couldn't start.
#[derive(Debug, thiserror::Error)]
pub enum StartupError {
    /// The database couldn't be opened.
    #[error("cannot open the database: {0}")]
    Database(#[from] sqlx::Error),
    /// A migration failed (or the database is newer than this program).
    #[error("cannot migrate the database: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    /// A setting the auth code rejected.
    #[error("bad auth settings: {0}")]
    Auth(String),
}

impl From<AppError> for StartupError {
    fn from(err: AppError) -> Self {
        StartupError::Auth(err.to_string())
    }
}

/// Opens the pool, as in lesson 06/03: an in-memory database must keep one
/// connection open forever, or it vanishes.
pub async fn connect(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(database_url)?;
    let in_memory = database_url.contains(":memory:") || database_url.contains("mode=memory");
    let mut pool = SqlitePoolOptions::new()
        .max_connections(if in_memory { 4 } else { 8 })
        .acquire_timeout(std::time::Duration::from_secs(5));
    if in_memory {
        pool = pool
            .min_connections(1)
            .idle_timeout(None)
            .max_lifetime(None);
    }
    pool.connect_with(options).await
}

impl AppState {
    /// Connects, migrates and wires the layers together:
    /// repositories (SQL) → services (rules) → state (for the handlers).
    pub async fn build(config: Config) -> Result<AppState, StartupError> {
        let db = connect(&config.database_url).await?;
        MIGRATOR.run(&db).await?;

        let passwords = Passwords::new(config.password_memory_kib, config.password_iterations)
            .map_err(|err| StartupError::Auth(format!("Argon2 parameters: {err}")))?;
        let tokens = Tokens::new(&config.jwt_secret, &config.jwt_issuer, config.token_ttl)?;

        let auth = AuthService::new(
            UserRepository::new(db.clone()),
            passwords,
            tokens,
            config.admin_emails.clone(),
        );
        let reviews = ReviewService::new(ReviewRepository::new(db.clone()));
        Ok(AppState {
            config: Arc::new(config),
            db,
            auth,
            reviews,
        })
    }
}
