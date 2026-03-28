use thiserror::Error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Core error type for TurtleShare.
//
// // TurtleShare 的核心错误类型。
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Too many requests: {0}")]
    TooManyRequests(String),

    #[error("Hashing error: {0}")]
    Hash(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Invalid hash ID: {0}")]
    InvalidHashId(String),
}

/// A specialized Result type for TurtleShare.
//
// // TurtleShare 的特化 Result 类型。
pub type Result<T> = std::result::Result<T, AppError>;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AppError::Config(m) => (StatusCode::INTERNAL_SERVER_ERROR, "CONFIG_ERROR", m),
            AppError::Database(m) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", m),
            AppError::Auth(m) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", m),
            AppError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", m),
            AppError::Forbidden(m) => (StatusCode::FORBIDDEN, "FORBIDDEN", m),
            AppError::ValidationError(m) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", m),
            AppError::TooManyRequests(m) => (StatusCode::TOO_MANY_REQUESTS, "TOO_MANY_REQUESTS", m),
            AppError::Hash(m) => (StatusCode::INTERNAL_SERVER_ERROR, "HASH_ERROR", m),
            AppError::Network(m) => (StatusCode::BAD_GATEWAY, "NETWORK_ERROR", m),
            AppError::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", m),
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, "NOT_FOUND", m),
            AppError::InvalidHashId(m) => (StatusCode::BAD_REQUEST, "INVALID_HASH_ID", m),
        };

        let body = Json(json!({
            "success": false,
            "error": {
                "code": code,
                "message": message
            }
        }));

        (status, body).into_response()
    }
}
