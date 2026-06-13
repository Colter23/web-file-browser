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
    UnsupportedMediaType(String),
    RangeNotSatisfiable(String),
    PreconditionRequired(String),
    PreconditionFailed(String),
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

    pub fn unsupported_media_type(message: impl Into<String>) -> Self {
        Self::UnsupportedMediaType(message.into())
    }

    pub fn range_not_satisfiable(message: impl Into<String>) -> Self {
        Self::RangeNotSatisfiable(message.into())
    }

    pub fn precondition_required(message: impl Into<String>) -> Self {
        Self::PreconditionRequired(message.into())
    }

    pub fn precondition_failed(message: impl Into<String>) -> Self {
        Self::PreconditionFailed(message.into())
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
            Self::UnsupportedMediaType(message) => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "UNSUPPORTED_MEDIA_TYPE",
                message,
            ),
            Self::RangeNotSatisfiable(message) => (
                StatusCode::RANGE_NOT_SATISFIABLE,
                "RANGE_NOT_SATISFIABLE",
                message,
            ),
            Self::PreconditionRequired(message) => (
                StatusCode::PRECONDITION_REQUIRED,
                "PRECONDITION_REQUIRED",
                message,
            ),
            Self::PreconditionFailed(message) => (
                StatusCode::PRECONDITION_FAILED,
                "PRECONDITION_FAILED",
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
            | Self::UnsupportedMediaType(message)
            | Self::RangeNotSatisfiable(message)
            | Self::PreconditionRequired(message)
            | Self::PreconditionFailed(message)
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

impl From<zip::result::ZipError> for AppError {
    fn from(error: zip::result::ZipError) -> Self {
        match error {
            zip::result::ZipError::FileNotFound => Self::NotFound("压缩包内路径不存在".to_string()),
            zip::result::ZipError::InvalidArchive(_)
            | zip::result::ZipError::UnsupportedArchive(_)
            | zip::result::ZipError::InvalidPassword => {
                Self::BadRequest("ZIP 压缩包无效或不受支持".to_string())
            }
            zip::result::ZipError::Io(error) => Self::Internal(error.to_string()),
            _ => Self::BadRequest("ZIP 压缩包无效或不受支持".to_string()),
        }
    }
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    code: &'static str,
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use serde_json::Value;

    #[tokio::test]
    async fn app_error_returns_stable_status_and_code() {
        let cases = [
            (
                AppError::bad_request("请求参数无效"),
                StatusCode::BAD_REQUEST,
                "BAD_REQUEST",
            ),
            (
                AppError::unauthorized("请先登录"),
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
            ),
            (
                AppError::forbidden("挂载点是只读模式"),
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
            ),
            (
                AppError::not_found("查无此路径"),
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
            ),
            (
                AppError::conflict("路径已存在"),
                StatusCode::CONFLICT,
                "CONFLICT",
            ),
            (
                AppError::too_many_requests("请求过多"),
                StatusCode::TOO_MANY_REQUESTS,
                "TOO_MANY_REQUESTS",
            ),
            (
                AppError::payload_too_large("上传内容过大"),
                StatusCode::PAYLOAD_TOO_LARGE,
                "PAYLOAD_TOO_LARGE",
            ),
            (
                AppError::unsupported_media_type("不支持在线编辑此文件"),
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "UNSUPPORTED_MEDIA_TYPE",
            ),
            (
                AppError::range_not_satisfiable("Range 起点越界"),
                StatusCode::RANGE_NOT_SATISFIABLE,
                "RANGE_NOT_SATISFIABLE",
            ),
            (
                AppError::precondition_required("保存文件需要 If-Match 头"),
                StatusCode::PRECONDITION_REQUIRED,
                "PRECONDITION_REQUIRED",
            ),
            (
                AppError::precondition_failed("文件已被外部修改"),
                StatusCode::PRECONDITION_FAILED,
                "PRECONDITION_FAILED",
            ),
            (
                AppError::internal("内部错误"),
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
            ),
        ];

        for (error, expected_status, expected_code) in cases {
            let response = error.into_response();
            assert_eq!(response.status(), expected_status);

            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let json: Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(json["code"], expected_code);
            assert!(
                json["message"]
                    .as_str()
                    .is_some_and(|message| !message.is_empty())
            );
        }
    }
}
