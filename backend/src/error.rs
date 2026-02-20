use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Post not found")]
    NotFound,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Internal Server Error")]
    Anyhow(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(ref e) => {
                println!("Database Error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong")
            }
            AppError::NotFound => (StatusCode::NOT_FOUND, "Resource not found"),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized Access"),
            AppError::Anyhow(ref e) => {
                println!("System Error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong")
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
