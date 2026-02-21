use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use crate::extractors::SiteIdentity;

pub struct AppError {
    pub status: StatusCode,
    pub message: Option<String>,
    pub debug: Option<String>,
    pub is_local: bool,
}

#[derive(Serialize)]
struct ErrorBody {
    code: u16,
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    debug: Option<String>,
}

impl AppError {
    /// Create a new error with just a status code.
    /// The message will default to the standard HTTP reason (e.g. "Not Found")
    pub fn new(status: StatusCode) -> Self {
        Self {
            status,
            message: None,
            debug: None,
            is_local: false,
        }
    }

    /// Set a custom user-facing message
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Set internal debug information (the "real" error)
    pub fn with_debug(mut self, debug: impl Into<String>) -> Self {
        self.debug = Some(debug.into());
        self
    }

    /// Check site identity to determine if we can show debug info
    pub fn at_site(mut self, site: &SiteIdentity) -> Self {
        self.is_local = site.domain.starts_with("localhost") || site.domain.starts_with("127.0.0.1");
        self
    }

    // Common shortcuts
    pub fn bad_request() -> Self {
        Self::new(StatusCode::BAD_REQUEST)
    }

    pub fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND)
    }

    pub fn unauthorized() -> Self {
        Self::new(StatusCode::UNAUTHORIZED)
    }
}

// Allow automatic conversion from SQL errors
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR)
            .with_message("A database error occurred")
            .with_debug(err.to_string())
    }
}

// Allow automatic conversion from anyhow::Error
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR)
            .with_message("An unexpected system error occurred")
            .with_debug(err.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status_name = self.status.canonical_reason().unwrap_or("Unknown");
        
        let body = Json(ErrorBody {
            code: self.status.as_u16(),
            status: status_name,
            message: self.message,
            debug: if self.is_local { self.debug } else { None },
        });

        (self.status, body).into_response()
    }
}
