//! `GET /health`: is the server up, and can it reach its database?
//!
//! Load balancers and container orchestrators call this to decide whether to
//! send traffic here (and the Docker `HEALTHCHECK` calls it too).

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde_json::{Value, json};

use crate::state::AppState;

/// 200 when the database answers `SELECT 1`, 503 when it doesn't.
pub async fn health(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    let database = sqlx::query_scalar::<_, i64>("SELECT 1")
        .fetch_one(&state.db)
        .await;
    let version = env!("CARGO_PKG_VERSION");
    match database {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "status": "ok", "database": "ok", "version": version })),
        ),
        Err(err) => {
            tracing::error!(%err, "health check: database unreachable");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(
                    json!({ "status": "degraded", "database": "unreachable", "version": version }),
                ),
            )
        }
    }
}
