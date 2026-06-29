use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_json::{Map, Value};
use std::{borrow::Cow, fmt, io};

#[derive(Debug)]
pub enum AppError {
    BadRequest(ErrorInfo),
    Unauthorized(ErrorInfo),
    Forbidden(ErrorInfo),
    NotFound(ErrorInfo),
    MethodNotAllowed(ErrorInfo),
    Conflict(ErrorInfo),
    TooManyRequests(ErrorInfo),
    PayloadTooLarge(ErrorInfo),
    UnsupportedMediaType(ErrorInfo),
    RangeNotSatisfiable(ErrorInfo),
    PreconditionRequired(ErrorInfo),
    PreconditionFailed(ErrorInfo),
    Internal(ErrorInfo),
}

impl AppError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(ErrorInfo::new(message))
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::Unauthorized(ErrorInfo::new(message))
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::Forbidden(ErrorInfo::new(message))
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(ErrorInfo::new(message))
    }

    pub fn method_not_allowed(message: impl Into<String>) -> Self {
        Self::MethodNotAllowed(ErrorInfo::new(message))
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict(ErrorInfo::new(message))
    }

    pub fn too_many_requests(message: impl Into<String>) -> Self {
        Self::TooManyRequests(ErrorInfo::new(message))
    }

    pub fn payload_too_large(message: impl Into<String>) -> Self {
        Self::PayloadTooLarge(ErrorInfo::new(message))
    }

    pub fn unsupported_media_type(message: impl Into<String>) -> Self {
        Self::UnsupportedMediaType(ErrorInfo::new(message))
    }

    pub fn range_not_satisfiable(message: impl Into<String>) -> Self {
        Self::RangeNotSatisfiable(ErrorInfo::new(message))
    }

    pub fn precondition_required(message: impl Into<String>) -> Self {
        Self::PreconditionRequired(ErrorInfo::new(message))
    }

    pub fn precondition_failed(message: impl Into<String>) -> Self {
        Self::PreconditionFailed(ErrorInfo::new(message))
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(ErrorInfo::new(message))
    }

    pub fn with_reason(self, reason: impl Into<Cow<'static, str>>) -> Self {
        self.map_info(|info| info.with_reason(reason))
    }

    pub fn with_param(self, key: impl Into<String>, value: impl Serialize) -> Self {
        self.map_info(|info| info.with_param(key, value))
    }

    pub fn code(&self) -> &'static str {
        self.status_code_reason_and_info().1
    }

    pub fn reason(&self) -> &str {
        self.info().reason.as_deref().unwrap_or_else(|| self.code())
    }

    pub fn params(&self) -> Option<&Map<String, Value>> {
        self.info().params.as_ref()
    }

    fn info(&self) -> &ErrorInfo {
        match self {
            Self::BadRequest(info)
            | Self::Unauthorized(info)
            | Self::Forbidden(info)
            | Self::NotFound(info)
            | Self::MethodNotAllowed(info)
            | Self::Conflict(info)
            | Self::TooManyRequests(info)
            | Self::PayloadTooLarge(info)
            | Self::UnsupportedMediaType(info)
            | Self::RangeNotSatisfiable(info)
            | Self::PreconditionRequired(info)
            | Self::PreconditionFailed(info)
            | Self::Internal(info) => info,
        }
    }

    fn map_info(self, mapper: impl FnOnce(ErrorInfo) -> ErrorInfo) -> Self {
        match self {
            Self::BadRequest(info) => Self::BadRequest(mapper(info)),
            Self::Unauthorized(info) => Self::Unauthorized(mapper(info)),
            Self::Forbidden(info) => Self::Forbidden(mapper(info)),
            Self::NotFound(info) => Self::NotFound(mapper(info)),
            Self::MethodNotAllowed(info) => Self::MethodNotAllowed(mapper(info)),
            Self::Conflict(info) => Self::Conflict(mapper(info)),
            Self::TooManyRequests(info) => Self::TooManyRequests(mapper(info)),
            Self::PayloadTooLarge(info) => Self::PayloadTooLarge(mapper(info)),
            Self::UnsupportedMediaType(info) => Self::UnsupportedMediaType(mapper(info)),
            Self::RangeNotSatisfiable(info) => Self::RangeNotSatisfiable(mapper(info)),
            Self::PreconditionRequired(info) => Self::PreconditionRequired(mapper(info)),
            Self::PreconditionFailed(info) => Self::PreconditionFailed(mapper(info)),
            Self::Internal(info) => Self::Internal(mapper(info)),
        }
    }

    fn status_code_reason_and_info(&self) -> (StatusCode, &'static str, &ErrorInfo) {
        match self {
            Self::BadRequest(info) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", info),
            Self::Unauthorized(info) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", info),
            Self::Forbidden(info) => (StatusCode::FORBIDDEN, "FORBIDDEN", info),
            Self::NotFound(info) => (StatusCode::NOT_FOUND, "NOT_FOUND", info),
            Self::MethodNotAllowed(info) => {
                (StatusCode::METHOD_NOT_ALLOWED, "METHOD_NOT_ALLOWED", info)
            }
            Self::Conflict(info) => (StatusCode::CONFLICT, "CONFLICT", info),
            Self::TooManyRequests(info) => {
                (StatusCode::TOO_MANY_REQUESTS, "TOO_MANY_REQUESTS", info)
            }
            Self::PayloadTooLarge(info) => {
                (StatusCode::PAYLOAD_TOO_LARGE, "PAYLOAD_TOO_LARGE", info)
            }
            Self::UnsupportedMediaType(info) => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "UNSUPPORTED_MEDIA_TYPE",
                info,
            ),
            Self::RangeNotSatisfiable(info) => (
                StatusCode::RANGE_NOT_SATISFIABLE,
                "RANGE_NOT_SATISFIABLE",
                info,
            ),
            Self::PreconditionRequired(info) => (
                StatusCode::PRECONDITION_REQUIRED,
                "PRECONDITION_REQUIRED",
                info,
            ),
            Self::PreconditionFailed(info) => {
                (StatusCode::PRECONDITION_FAILED, "PRECONDITION_FAILED", info)
            }
            Self::Internal(info) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", info),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, info) = self.status_code_reason_and_info();
        let reason = info.reason.as_deref().unwrap_or(code).to_string();

        (
            status,
            Json(ErrorResponse {
                code,
                reason,
                message: info.message.clone(),
                params: info.params.clone(),
            }),
        )
            .into_response()
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadRequest(message)
            | Self::Unauthorized(message)
            | Self::Forbidden(message)
            | Self::NotFound(message)
            | Self::MethodNotAllowed(message)
            | Self::Conflict(message)
            | Self::TooManyRequests(message)
            | Self::PayloadTooLarge(message)
            | Self::UnsupportedMediaType(message)
            | Self::RangeNotSatisfiable(message)
            | Self::PreconditionRequired(message)
            | Self::PreconditionFailed(message)
            | Self::Internal(message) => formatter.write_str(&message.message),
        }
    }
}

impl std::error::Error for AppError {}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        Self::internal(error.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        Self::internal(error.to_string())
    }
}

impl From<tokio::task::JoinError> for AppError {
    fn from(error: tokio::task::JoinError) -> Self {
        Self::internal(error.to_string())
    }
}

impl From<axum::extract::multipart::MultipartError> for AppError {
    fn from(error: axum::extract::multipart::MultipartError) -> Self {
        Self::bad_request(error.to_string()).with_reason("MULTIPART_ERROR")
    }
}

impl From<zip::result::ZipError> for AppError {
    fn from(error: zip::result::ZipError) -> Self {
        match error {
            zip::result::ZipError::FileNotFound => {
                Self::not_found("压缩包内路径不存在").with_reason("ARCHIVE_ENTRY_NOT_FOUND")
            }
            zip::result::ZipError::InvalidArchive(_)
            | zip::result::ZipError::UnsupportedArchive(_)
            | zip::result::ZipError::InvalidPassword => {
                Self::bad_request("ZIP 压缩包无效或不受支持").with_reason("ZIP_ARCHIVE_INVALID")
            }
            zip::result::ZipError::Io(error) => Self::internal(error.to_string()),
            _ => Self::bad_request("ZIP 压缩包无效或不受支持").with_reason("ZIP_ARCHIVE_INVALID"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ErrorInfo {
    message: String,
    reason: Option<Cow<'static, str>>,
    params: Option<Map<String, Value>>,
}

impl ErrorInfo {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            reason: None,
            params: None,
        }
    }

    fn with_reason(mut self, reason: impl Into<Cow<'static, str>>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    fn with_param(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        let value = serde_json::to_value(value).unwrap_or(Value::Null);
        self.params
            .get_or_insert_with(Map::new)
            .insert(key.into(), value);
        self
    }
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    code: &'static str,
    reason: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Map<String, Value>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use serde_json::Value;

    #[tokio::test]
    async fn app_error_returns_stable_status_code_and_reason() {
        let cases = [
            (
                AppError::bad_request("请求参数无效"),
                StatusCode::BAD_REQUEST,
                "BAD_REQUEST",
                "BAD_REQUEST",
            ),
            (
                AppError::unauthorized("请先登录"),
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "UNAUTHORIZED",
            ),
            (
                AppError::forbidden("挂载点是只读模式"),
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "FORBIDDEN",
            ),
            (
                AppError::not_found("查无此路径"),
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                "NOT_FOUND",
            ),
            (
                AppError::method_not_allowed("请求方法不支持"),
                StatusCode::METHOD_NOT_ALLOWED,
                "METHOD_NOT_ALLOWED",
                "METHOD_NOT_ALLOWED",
            ),
            (
                AppError::conflict("路径已存在"),
                StatusCode::CONFLICT,
                "CONFLICT",
                "CONFLICT",
            ),
            (
                AppError::too_many_requests("请求过多"),
                StatusCode::TOO_MANY_REQUESTS,
                "TOO_MANY_REQUESTS",
                "TOO_MANY_REQUESTS",
            ),
            (
                AppError::payload_too_large("上传内容过大"),
                StatusCode::PAYLOAD_TOO_LARGE,
                "PAYLOAD_TOO_LARGE",
                "PAYLOAD_TOO_LARGE",
            ),
            (
                AppError::unsupported_media_type("不支持在线编辑此文件"),
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "UNSUPPORTED_MEDIA_TYPE",
                "UNSUPPORTED_MEDIA_TYPE",
            ),
            (
                AppError::range_not_satisfiable("Range 起点越界"),
                StatusCode::RANGE_NOT_SATISFIABLE,
                "RANGE_NOT_SATISFIABLE",
                "RANGE_NOT_SATISFIABLE",
            ),
            (
                AppError::precondition_required("保存文件需要 If-Match 头"),
                StatusCode::PRECONDITION_REQUIRED,
                "PRECONDITION_REQUIRED",
                "PRECONDITION_REQUIRED",
            ),
            (
                AppError::precondition_failed("文件已被外部修改"),
                StatusCode::PRECONDITION_FAILED,
                "PRECONDITION_FAILED",
                "PRECONDITION_FAILED",
            ),
            (
                AppError::internal("内部错误"),
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "INTERNAL_ERROR",
            ),
        ];

        for (error, expected_status, expected_code, expected_reason) in cases {
            let response = error.into_response();
            assert_eq!(response.status(), expected_status);

            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let json: Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(json["code"], expected_code);
            assert_eq!(json["reason"], expected_reason);
            assert!(
                json["message"]
                    .as_str()
                    .is_some_and(|message| !message.is_empty())
            );
        }
    }

    #[tokio::test]
    async fn app_error_returns_custom_reason_and_params() {
        let response = AppError::bad_request("分页大小必须大于 0")
            .with_reason("PAGE_SIZE_MUST_BE_POSITIVE")
            .with_param("field", "limit")
            .into_response();

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["code"], "BAD_REQUEST");
        assert_eq!(json["reason"], "PAGE_SIZE_MUST_BE_POSITIVE");
        assert_eq!(json["message"], "分页大小必须大于 0");
        assert_eq!(json["params"]["field"], "limit");
    }
}
