//! One error type for every layer, and one JSON shape for every error:
//!
//! ```json
//! {"error": {"code": "validation_failed", "message": "…", "details": [{"field": "email", "message": "…"}]}}
//! ```
//!
//! Repositories return `sqlx::Error`; services turn what they understand into
//! `AppError` variants and let `?` turn the rest into `Internal`; handlers
//! return `Result<_, AppError>`, and `IntoResponse` below picks the status.

use axum::Json;
use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::json;

/// One problem with one field of the input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FieldError {
    /// The JSON field, e.g. `"email"`.
    pub field: &'static str,
    /// What's wrong, for a person to read.
    pub message: String,
}

impl FieldError {
    /// A problem with `field`.
    pub fn new(field: &'static str, message: impl Into<String>) -> FieldError {
        FieldError {
            field,
            message: message.into(),
        }
    }
}

/// Everything that can go wrong while answering a request.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// 422: well-formed input that breaks the rules.
    #[error("the request has invalid fields")]
    Validation(Vec<FieldError>),
    /// 400: a request that makes no sense (e.g. `min_rating=9`).
    #[error("{0}")]
    BadRequest(String),
    /// An extractor refused the request before the handler ran: unreadable
    /// JSON (400), a wrong content type (415), JSON of the wrong shape (422),
    /// a body over the limit (413), a bad path or query (400). We keep the
    /// status axum chose and only change the format.
    #[error("{message}")]
    Rejected {
        /// The status axum's rejection carries.
        status: StatusCode,
        /// Axum's explanation.
        message: String,
    },
    /// 401: no credentials, or wrong ones.
    #[error("{0}")]
    Unauthorized(&'static str),
    /// 403: we know who you are, and you may not do this.
    #[error("you are not allowed to do this")]
    Forbidden,
    /// 404: no such thing.
    #[error("{0} not found")]
    NotFound(String),
    /// 405: the path exists, but not with this method.
    #[error("{0}")]
    MethodNotAllowed(String),
    /// 409: it clashes with what's there, e.g. an e-mail already registered.
    #[error("{0}")]
    Conflict(String),
    /// 500: our fault. The details are logged, never sent to the client.
    #[error("internal error: {0}")]
    Internal(String),
}

impl AppError {
    /// The HTTP status for this error.
    pub fn status(&self) -> StatusCode {
        match self {
            AppError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Rejected { status, .. } => *status,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::MethodNotAllowed(_) => StatusCode::METHOD_NOT_ALLOWED,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// A stable, machine-readable code for clients to match on.
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Validation(_) => "validation_failed",
            AppError::BadRequest(_) => "bad_request",
            AppError::Rejected { status, .. } => match status.as_u16() {
                413 => "payload_too_large",
                415 => "unsupported_media_type",
                422 => "invalid_body",
                _ => "bad_request",
            },
            AppError::Unauthorized(_) => "unauthorized",
            AppError::Forbidden => "forbidden",
            AppError::NotFound(_) => "not_found",
            AppError::MethodNotAllowed(_) => "method_not_allowed",
            AppError::Conflict(_) => "conflict",
            AppError::Internal(_) => "internal",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        let message = match &self {
            AppError::Internal(details) => {
                // Log the real cause; tell the client nothing useful to an attacker.
                tracing::error!(%details, "internal error");
                "something went wrong on our side".to_string()
            }
            other => other.to_string(),
        };
        let mut body = json!({ "code": self.code(), "message": message });
        if let AppError::Validation(details) = &self {
            body["details"] = json!(details);
        }
        let mut response = (status, Json(json!({ "error": body }))).into_response();
        if status == StatusCode::UNAUTHORIZED {
            response
                .headers_mut()
                .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        }
        response
    }
}

/// Database errors the service layer didn't handle become 500s.
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Internal(format!("database: {err}"))
    }
}

/// So that `AppJson<T>` (see `handlers`) answers bad JSON in our format.
impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        AppError::Rejected {
            status: rejection.status(),
            message: rejection.body_text(),
        }
    }
}

impl From<QueryRejection> for AppError {
    fn from(rejection: QueryRejection) -> Self {
        AppError::Rejected {
            status: rejection.status(),
            message: rejection.body_text(),
        }
    }
}

impl From<PathRejection> for AppError {
    fn from(rejection: PathRejection) -> Self {
        AppError::Rejected {
            status: rejection.status(),
            message: rejection.body_text(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statuses_and_codes() {
        assert_eq!(AppError::Forbidden.status(), StatusCode::FORBIDDEN);
        assert_eq!(
            AppError::NotFound("review 3".into()).to_string(),
            "review 3 not found"
        );
        assert_eq!(AppError::Validation(vec![]).code(), "validation_failed");
    }

    #[test]
    fn internal_errors_hide_their_details() {
        let response = AppError::Internal("disk on fire".into()).into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn unauthorized_says_how_to_authenticate() {
        let response = AppError::Unauthorized("missing token").into_response();
        assert_eq!(response.headers()[header::WWW_AUTHENTICATE], "Bearer");
    }
}
