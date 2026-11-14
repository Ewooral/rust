//! Registration, login and token checks.

use crate::auth::{Claims, Passwords, Tokens};
use crate::domain::{LoginRequest, RegisterRequest, Role, TokenResponse, User};
use crate::error::AppError;
use crate::repository::UserRepository;

/// Everything about accounts and credentials.
#[derive(Clone)]
pub struct AuthService {
    users: UserRepository,
    passwords: Passwords,
    tokens: Tokens,
    admin_emails: Vec<String>,
}

impl AuthService {
    /// Builds the service from its parts.
    pub fn new(
        users: UserRepository,
        passwords: Passwords,
        tokens: Tokens,
        admin_emails: Vec<String>,
    ) -> Self {
        AuthService {
            users,
            passwords,
            tokens,
            admin_emails,
        }
    }

    /// Validates, hashes the password, stores the user. Addresses listed in
    /// `ADMIN_EMAILS` become admins.
    pub async fn register(&self, request: RegisterRequest) -> Result<User, AppError> {
        let request = request.validate().map_err(AppError::Validation)?;
        let role = if self.admin_emails.contains(&request.email) {
            Role::Admin
        } else {
            Role::Member
        };

        // Argon2 takes tens of milliseconds of CPU on purpose. Doing that on
        // an async worker thread would stall every other request on it, so
        // it runs on Tokio's blocking thread pool instead.
        let passwords = self.passwords.clone();
        let password = request.password.clone();
        let hash = tokio::task::spawn_blocking(move || passwords.hash(&password))
            .await
            .map_err(|err| AppError::Internal(format!("hashing task: {err}")))??;

        match self
            .users
            .insert(&request.email, &request.display_name, &hash, role)
            .await
        {
            Ok(user) => {
                tracing::info!(user_id = user.id, role = role.as_str(), "registered");
                Ok(user)
            }
            Err(err) if is_unique_violation(&err) => Err(AppError::Conflict(
                "an account with this e-mail address already exists".into(),
            )),
            Err(err) => Err(err.into()),
        }
    }

    /// Checks the e-mail and password and returns a fresh access token.
    /// The error never says WHICH of the two was wrong.
    pub async fn login(&self, request: LoginRequest) -> Result<TokenResponse, AppError> {
        const WRONG: AppError = AppError::Unauthorized("wrong e-mail address or password");
        let email = request.email.trim().to_lowercase();
        let stored = self.users.find_by_email(&email).await?;

        let passwords = self.passwords.clone();
        let password = request.password;
        let hash = stored.as_ref().map(|s| s.password_hash.clone());
        let matches = tokio::task::spawn_blocking(move || match hash {
            Some(hash) => passwords.verify(&password, &hash),
            None => {
                passwords.verify_dummy(&password); // same time as a real check
                false
            }
        })
        .await
        .map_err(|err| AppError::Internal(format!("verifying task: {err}")))?;

        let Some(stored) = stored.filter(|_| matches) else {
            tracing::warn!("failed login");
            return Err(WRONG);
        };
        let access_token = self.tokens.issue(stored.user.id, stored.user.role)?;
        tracing::info!(user_id = stored.user.id, "logged in");
        Ok(TokenResponse {
            access_token,
            token_type: "Bearer".into(),
            expires_in: self.tokens.ttl().as_secs(),
        })
    }

    /// Checks a bearer token (used by the `AuthUser` extractor).
    pub fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
        self.tokens.verify(token)
    }

    /// The account behind a token.
    pub async fn me(&self, user_id: i64) -> Result<User, AppError> {
        self.users
            .find_by_id(user_id)
            .await?
            // A valid token for a deleted account.
            .ok_or(AppError::Unauthorized("this account no longer exists"))
    }
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    err.as_database_error()
        .is_some_and(|db| db.is_unique_violation())
}
