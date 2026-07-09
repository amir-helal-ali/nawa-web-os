//! Structured error handling — consistent JSON error responses.

#![allow(dead_code)]

use nawa_http::{Response, StatusCode};

/// Application error types.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum AppError {
    /// 400 — Bad request (malformed input).
    BadRequest { message: String, field: Option<String> },
    /// 401 — Authentication required or failed.
    Unauthorized { message: String },
    /// 403 — Authenticated but not permitted.
    Forbidden { message: String, required_role: Option<String> },
    /// 404 — Resource not found.
    NotFound { resource: String, id: String },
    /// 409 — Conflict (duplicate, version mismatch).
    Conflict { message: String, resource: String },
    /// 422 — Validation error.
    Validation { errors: Vec<FieldError> },
    /// 429 — Rate limit exceeded.
    RateLimited { retry_after_secs: u64 },
    /// 500 — Internal server error.
    Internal { message: String, error_id: String },
    /// 503 — Service unavailable (health check failed).
    ServiceUnavailable { service: String, message: String },
}

/// Field-level validation error.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
    pub code: String,
}

impl AppError {
    /// Get the HTTP status code for this error.
    pub fn status_code(&self) -> u16 {
        match self {
            AppError::BadRequest { .. } => 400,
            AppError::Unauthorized { .. } => 401,
            AppError::Forbidden { .. } => 403,
            AppError::NotFound { .. } => 404,
            AppError::Conflict { .. } => 409,
            AppError::Validation { .. } => 422,
            AppError::RateLimited { .. } => 429,
            AppError::Internal { .. } => 500,
            AppError::ServiceUnavailable { .. } => 503,
        }
    }

    /// Convert to an HTTP response with JSON body.
    pub fn to_response(&self) -> Response {
        let status = self.status_code();
        let body = serde_json::json!({
            "error": self,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        let mut r = Response::new(StatusCode(status));
        r.header("Content-Type", "application/json; charset=utf-8");
        r.body = serde_json::to_vec(&body).unwrap_or_default();
        if let AppError::RateLimited { retry_after_secs } = self {
            r.header("Retry-After", &retry_after_secs.to_string());
        }
        r
    }

    /// Quick constructors.
    pub fn bad_request(msg: impl Into<String>) -> Self {
        AppError::BadRequest { message: msg.into(), field: None }
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        AppError::Unauthorized { message: msg.into() }
    }

    pub fn forbidden(msg: impl Into<String>) -> Self {
        AppError::Forbidden { message: msg.into(), required_role: None }
    }

    pub fn not_found(resource: impl Into<String>, id: impl Into<String>) -> Self {
        AppError::NotFound { resource: resource.into(), id: id.into() }
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        let error_id = generate_error_id();
        AppError::Internal { message: msg.into(), error_id }
    }
}

/// Generate a unique error ID for tracing.
fn generate_error_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    format!("err_{:08x}", nanos)
}

/// Result type alias for convenience.
pub type AppResult<T> = Result<T, AppError>;

/// Helper to convert `AppError` into a response in route handlers.
pub fn handle_error(e: AppError) -> Response {
    tracing::warn!("Request error: {:?}", e);
    e.to_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_codes_are_correct() {
        assert_eq!(AppError::bad_request("test").status_code(), 400);
        assert_eq!(AppError::unauthorized("test").status_code(), 401);
        assert_eq!(AppError::forbidden("test").status_code(), 403);
        assert_eq!(AppError::not_found("User", "42").status_code(), 404);
        assert_eq!(AppError::internal("test").status_code(), 500);
    }

    #[test]
    fn error_response_has_json_body() {
        let err = AppError::bad_request("invalid email");
        let resp = err.to_response();
        assert_eq!(resp.status.0, 400);
        assert!(resp.headers.iter().any(|(k, _)| k == "Content-Type"));
        let body = String::from_utf8(resp.body.clone()).unwrap();
        assert!(body.contains("BadRequest"));
        assert!(body.contains("invalid email"));
    }

    #[test]
    fn rate_limited_has_retry_after_header() {
        let err = AppError::RateLimited { retry_after_secs: 60 };
        let resp = err.to_response();
        assert_eq!(resp.status.0, 429);
        assert!(resp.headers.iter().any(|(k, v)| k == "Retry-After" && v == "60"));
    }

    #[test]
    fn internal_error_has_error_id() {
        let err = AppError::internal("database failed");
        let resp = err.to_response();
        let body = String::from_utf8(resp.body.clone()).unwrap();
        assert!(body.contains("error_id"));
        assert!(body.contains("err_"));
    }

    #[test]
    fn error_serializes_to_json() {
        let err = AppError::not_found("User", "42");
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("NotFound"));
        assert!(json.contains("User"));
        assert!(json.contains("42"));
    }

    #[test]
    fn validation_error_has_multiple_fields() {
        let err = AppError::Validation {
            errors: vec![
                FieldError { field: "email".into(), message: "invalid".into(), code: "INVALID".into() },
                FieldError { field: "password".into(), message: "too short".into(), code: "TOO_SHORT".into() },
            ]
        };
        let resp = err.to_response();
        assert_eq!(resp.status.0, 422);
        let body = String::from_utf8(resp.body.clone()).unwrap();
        assert!(body.contains("email"));
        assert!(body.contains("password"));
    }
}
