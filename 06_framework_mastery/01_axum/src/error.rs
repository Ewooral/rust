//! One error type for the whole API. Handlers return
//! `Result<_, ApiError>`, use `?` freely, and this file decides how every
//! failure looks on the wire: a status code plus a JSON body such as
//! `{"error":"note 7 not found"}`.

use axum::Json;
use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    /// 404: no note with this id.
    NotFound(u64),
    /// 404: no route matched, e.g. `"GET /nope"`.
    NoRoute(String),
    /// 422: the JSON was well formed, but broke the rules.
    Validation(Vec<String>),
    /// An extractor refused the request (bad JSON, wrong content type,
    /// a path id that isn't a number...). We keep the status axum chose.
    Rejected { status: StatusCode, message: String },
}

impl ApiError {
    pub fn status(&self) -> StatusCode {
        match self {
            ApiError::NotFound(_) | ApiError::NoRoute(_) => StatusCode::NOT_FOUND,
            ApiError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::Rejected { status, .. } => *status,
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::NotFound(id) => write!(f, "note {id} not found"),
            ApiError::NoRoute(route) => write!(f, "no route for {route}"),
            ApiError::Validation(_) => write!(f, "validation failed"),
            ApiError::Rejected { message, .. } => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for ApiError {}

/// The heart of error handling in axum: turn the error into a `Response`.
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status();
        let body = match &self {
            ApiError::Validation(details) => {
                json!({ "error": self.to_string(), "details": details })
            }
            _ => json!({ "error": self.to_string() }),
        };
        tracing::debug!(%status, error = %self, "request failed");
        (status, Json(body)).into_response()
    }
}

// `From` impls let a handler write `payload?` on a `Result<Json<T>,
// JsonRejection>`: the `?` converts the rejection into an `ApiError`.

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        ApiError::Rejected {
            status: rejection.status(),
            message: rejection.body_text(),
        }
    }
}

impl From<PathRejection> for ApiError {
    fn from(rejection: PathRejection) -> Self {
        ApiError::Rejected {
            status: rejection.status(),
            message: rejection.body_text(),
        }
    }
}

impl From<QueryRejection> for ApiError {
    fn from(rejection: QueryRejection) -> Self {
        ApiError::Rejected {
            status: rejection.status(),
            message: rejection.body_text(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_variant_has_the_right_status() {
        assert_eq!(ApiError::NotFound(1).status(), StatusCode::NOT_FOUND);
        assert_eq!(
            ApiError::Validation(vec![]).status(),
            StatusCode::UNPROCESSABLE_ENTITY
        );
        let rejected = ApiError::Rejected {
            status: StatusCode::UNSUPPORTED_MEDIA_TYPE,
            message: "x".into(),
        };
        assert_eq!(rejected.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[test]
    fn into_response_sets_status_and_json_content_type() {
        let response = ApiError::NotFound(7).into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            response.headers()["content-type"],
            "application/json",
            "Json(..) sets the content type"
        );
    }
}
