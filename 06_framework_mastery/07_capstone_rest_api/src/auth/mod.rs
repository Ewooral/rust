//! Authentication: who is calling? Passwords, tokens, and the `AuthUser`
//! extractor that turns an `Authorization: Bearer <token>` header into a user.

pub mod password;
pub mod token;

use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;

use crate::domain::Role;
use crate::error::AppError;
use crate::state::AppState;

pub use password::Passwords;
pub use token::{Claims, Tokens};

/// The caller, proven by a valid token. Put it in a handler's arguments and
/// the handler only runs for authenticated requests; everyone else gets a 401
/// with a JSON body (the extractor's rejection is an `AppError`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthUser {
    /// The user's id (the token's `sub`).
    pub id: i64,
    /// The role the token was issued with.
    pub role: Role,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or(AppError::Unauthorized("missing Authorization header"))?;
        let token = header
            .to_str()
            .ok()
            .and_then(|value| value.strip_prefix("Bearer "))
            .map(str::trim)
            .filter(|token| !token.is_empty())
            .ok_or(AppError::Unauthorized(
                "expected `Authorization: Bearer <token>`",
            ))?;
        let claims = state.auth.verify_token(token)?;
        let id = claims
            .sub
            .parse()
            .map_err(|_| AppError::Unauthorized("the token is not valid"))?;
        Ok(AuthUser {
            id,
            role: claims.role,
        })
    }
}
