//! The handler layer: HTTP in, HTTP out.
//!
//! A handler extracts what it needs from the request, calls ONE service
//! method, and turns the result into a response. No SQL and no business
//! rules here: if a handler grows an `if`, that rule probably belongs in a
//! service.

pub mod auth;
pub mod health;
pub mod reviews;

use axum::extract::{FromRequest, FromRequestParts};
use axum::http::{Method, Uri};
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::error::AppError;

/// `axum::Json`, but a bad body is answered in OUR error format.
/// The derive forwards to `axum::Json` and converts its rejection with
/// `From<JsonRejection> for AppError`.
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(AppError))]
pub struct AppJson<T>(pub T);

impl<T: Serialize> IntoResponse for AppJson<T> {
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

/// `axum::extract::Query` with our error format.
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(AppError))]
pub struct AppQuery<T>(pub T);

/// `axum::extract::Path` with our error format.
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(AppError))]
pub struct AppPath<T>(pub T);

/// No route matched: a JSON 404 instead of axum's empty one.
pub async fn not_found(method: Method, uri: Uri) -> AppError {
    AppError::NotFound(format!("route {method} {}", uri.path()))
}

/// The path exists, the method doesn't: a JSON 405.
pub async fn method_not_allowed(method: Method, uri: Uri) -> AppError {
    AppError::MethodNotAllowed(format!("{method} is not allowed on {}", uri.path()))
}
