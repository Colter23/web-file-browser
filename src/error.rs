use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::{fmt, io};

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    TooManyRequests(String),
    PayloadTooLarge(String),
    RangeNotSatisfiable(String),
    Internal(String),
}

impl AppError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::Unauthorized(message.into())
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::Forbidden(message.into())
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict(message.into())
    }

    pub fn too_many_requests(message: impl Into<String>) -> Self {
        Self::TooManyRequests(message.into())
    }

    pub fn payload_too_large(message: impl Into<String>) -> Self {
        Self::PayloadTooLarge(message.into())
    }

    pub fn range_not_satisfiable(message: impl Into<String>) -> Self {
        Self::RangeNotSatisfiable(message.into())
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", message),
            Self::Unauthorized(message) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", message),
            Self::Forbidden(message) => (StatusCode::FORBIDDEN, "FORBIDDEN", message),
            Self::NotFound(message) => (StatusCode::NOT_FOUND, "NOT_FOUND", message),
            Self::Conflict(message) => (StatusCode::CONFLICT, "CONFLICT", message),
            Self::TooManyRequests(message) => {
                (StatusCode::TOO_MANY_REQUESTS, "TOO_MANY_REQUESTS", message)
            }
            Self::PayloadTooLarge(message) => {
                (StatusCode::PAYLOAD_TOO_LARGE, "PAYLOAD_TOO_LARGE", message)
            }
            Self::RangeNotSatisfiable(message) => (
                StatusCode::RANGE_NOT_SATISFIABLE,
                "RANGE_NOT_SATISFIABLE",
                message,
            ),
            Self::Internal(message) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", message)
            }
        };

        (status, Json(ErrorResponse { code, message })).into_response()
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadRequest(message)
            | Self::Unauthorized(message)
            | Self::Forbidden(message)
            | Self::NotFound(message)
            | Self::Conflict(message)
            | Self::TooManyRequests(message)
            | Self::PayloadTooLarge(message)
            | Self::RangeNotSatisfiable(message)
            | Self::Internal(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for AppError {}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        Self::Internal(error.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        Self::Internal(error.to_string())
    }
}

impl From<tokio::task::JoinError> for AppError {
    fn from(error: tokio::task::JoinError) -> Self {
        Self::Internal(error.to_string())
    }
}

impl From<axum::extract::multipart::MultipartError> for AppError {
    fn from(error: axum::extract::multipart::MultipartError) -> Self {
        Self::BadRequest(error.to_string())
    }
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    code: &'static str,
    message: String,
}
