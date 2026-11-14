//! `/api/v1/reviews` and `/api/v1/reviews/{id}`.

use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;

use crate::auth::AuthUser;
use crate::domain::{ListParams, NewReview, Page, Review, ReviewPatch};
use crate::error::AppError;
use crate::handlers::{AppJson, AppPath, AppQuery};
use crate::state::AppState;

/// `GET /api/v1/reviews?page=&per_page=&book=&min_rating=` (public).
pub async fn list(
    State(state): State<AppState>,
    AppQuery(params): AppQuery<ListParams>,
) -> Result<AppJson<Page<Review>>, AppError> {
    Ok(AppJson(state.reviews.list(&params).await?))
}

/// `POST /api/v1/reviews` (logged in) → 201, `Location`, and the review.
pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    AppJson(new): AppJson<NewReview>,
) -> Result<impl IntoResponse, AppError> {
    let review = state.reviews.create(&user, new).await?;
    let location = format!("/api/v1/reviews/{}", review.id);
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, location)],
        AppJson(review),
    ))
}

/// `GET /api/v1/reviews/{id}` (public).
pub async fn get(
    State(state): State<AppState>,
    AppPath(id): AppPath<i64>,
) -> Result<AppJson<Review>, AppError> {
    Ok(AppJson(state.reviews.get(id).await?))
}

/// `PATCH /api/v1/reviews/{id}` (the author, or an admin).
pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    AppPath(id): AppPath<i64>,
    AppJson(patch): AppJson<ReviewPatch>,
) -> Result<AppJson<Review>, AppError> {
    Ok(AppJson(state.reviews.update(&user, id, patch).await?))
}

/// `DELETE /api/v1/reviews/{id}` (the author, or an admin) → 204.
pub async fn delete(
    State(state): State<AppState>,
    user: AuthUser,
    AppPath(id): AppPath<i64>,
) -> Result<StatusCode, AppError> {
    state.reviews.delete(&user, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
