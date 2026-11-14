//! One error type for the whole API, turned into JSON by `ResponseError`.
//!
//! Handlers return `Result<_, ApiError>` and use `?` freely. In Axum the same
//! job is done by `impl IntoResponse for ApiError` (lesson 06/01); in Actix it
//! is `impl ResponseError`, which needs `Debug + Display` first.

use std::fmt;

use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde_json::json;

/// Everything the API can answer with an error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    /// 404: no note with this id.
    NotFound(u64),
    /// 404: no route matched, e.g. `"GET /nope"`.
    NoRoute(String),
    /// 422: the JSON was well formed but broke the rules.
    Validation(Vec<String>),
    /// An extractor refused the request (bad JSON, a path id that isn't a
    /// number, an unknown query parameter…). Keeps the status Actix chose.
    Rejected {
        /// Usually 400; 413 for a body that's too big.
        status: StatusCode,
        /// Actix's own explanation.
        message: String,
    },
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

/// How Actix turns the error into a response: a status code, and a body.
impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::NotFound(_) | ApiError::NoRoute(_) => StatusCode::NOT_FOUND,
            ApiError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::Rejected { status, .. } => *status,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let body = match self {
            ApiError::Validation(details) => {
                json!({ "error": self.to_string(), "details": details })
            }
            _ => json!({ "error": self.to_string() }),
        };
        HttpResponse::build(self.status_code()).json(body)
    }
}

impl ApiError {
    /// Wraps an extractor's error (`JsonPayloadError`, `PathError`,
    /// `QueryPayloadError`) so it is answered in our JSON format.
    pub fn rejected(err: &dyn ResponseError) -> ApiError {
        ApiError::Rejected {
            status: err.status_code(),
            message: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statuses_and_messages() {
        assert_eq!(ApiError::NotFound(7).status_code(), StatusCode::NOT_FOUND);
        assert_eq!(ApiError::NotFound(7).to_string(), "note 7 not found");
        let invalid = ApiError::Validation(vec!["title must not be empty".into()]);
        assert_eq!(invalid.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
