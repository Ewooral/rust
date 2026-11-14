//! `/api/v1/auth/*` and `/api/v1/me`.

use axum::extract::State;
use axum::http::StatusCode;

use crate::auth::AuthUser;
use crate::domain::{LoginRequest, RegisterRequest, TokenResponse, User};
use crate::error::AppError;
use crate::handlers::AppJson;
use crate::state::AppState;

/// `POST /api/v1/auth/register` → 201 and the new user (never the hash).
pub async fn register(
    State(state): State<AppState>,
    AppJson(request): AppJson<RegisterRequest>,
) -> Result<(StatusCode, AppJson<User>), AppError> {
    let user = state.auth.register(request).await?;
    Ok((StatusCode::CREATED, AppJson(user)))
}

/// `POST /api/v1/auth/login` → 200 and an access token.
pub async fn login(
    State(state): State<AppState>,
    AppJson(request): AppJson<LoginRequest>,
) -> Result<AppJson<TokenResponse>, AppError> {
    Ok(AppJson(state.auth.login(request).await?))
}

/// `GET /api/v1/me` → the caller's own account. Needs a token.
pub async fn me(State(state): State<AppState>, user: AuthUser) -> Result<AppJson<User>, AppError> {
    Ok(AppJson(state.auth.me(user.id).await?))
}
