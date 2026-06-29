mod audit;
mod auth;
mod favorites;
mod file;
mod mapping;
mod ops;
mod search;
mod settings;
mod tasks;
mod trash;

use axum::{
    Router,
    body::Body,
    extract::{ConnectInfo, Request, State},
    http::{StatusCode, header::CONTENT_TYPE},
    middleware,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use crate::{
    app::AppState,
    error::AppError,
    routes::{
        audit::audit_routes,
        auth::{protected_auth_routes, public_auth_routes, require_auth},
        favorites::favorite_routes,
        file::file_routes,
        mapping::mapping_routes,
        ops::{ops_routes, public_ops_routes},
        search::search_routes,
        settings::settings_routes,
        tasks::task_routes,
        trash::trash_routes,
    },
};

pub fn api_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let public_auth_routes =
        public_auth_routes().layer(middleware::from_fn_with_state(state.clone(), limit_ip));
    let protected_routes = Router::new()
        .merge(protected_auth_routes())
        .merge(audit_routes())
        .merge(favorite_routes())
        .merge(file_routes())
        .merge(mapping_routes())
        .merge(settings_routes())
        .merge(task_routes())
        .merge(search_routes())
        .merge(trash_routes())
        .merge(ops_routes())
        .fallback(api_not_found)
        .method_not_allowed_fallback(api_method_not_allowed)
        .layer(middleware::from_fn_with_state(state.clone(), require_auth))
        .layer(middleware::from_fn_with_state(state, limit_ip));

    Router::new()
        .merge(public_auth_routes)
        .merge(public_ops_routes())
        .merge(protected_routes)
        .layer(middleware::from_fn(normalize_api_error_response))
}

pub async fn api_index() -> &'static str {
    "Web File Browser Server"
}

pub async fn api_not_found() -> AppError {
    AppError::not_found("API 路径不存在").with_reason("API_ROUTE_NOT_FOUND")
}

pub async fn api_method_not_allowed() -> AppError {
    AppError::method_not_allowed("请求方法不支持").with_reason("METHOD_NOT_ALLOWED")
}

pub async fn normalize_api_error_response(request: Request, next: Next) -> Response {
    let response = next.run(request).await;
    if response.status().is_success()
        || response.status().is_redirection()
        || response.status() == StatusCode::NOT_MODIFIED
        || response.status() == StatusCode::NO_CONTENT
        || is_json_response(&response)
    {
        return response;
    }

    match generic_error_for_status(response.status()) {
        Some(error) => error.into_response(),
        None => response,
    }
}

pub async fn limit_ip(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = request_ip(
        &request,
        state.settings.active_startup().trust_proxy_headers,
    );
    let _permit = state.limits.acquire_ip(ip).await?;
    Ok(next.run(request).await)
}

fn request_ip(request: &Request, trust_proxy_headers: bool) -> String {
    if trust_proxy_headers
        && let Some(ip) = request
            .headers()
            .get("x-forwarded-for")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(',').next())
            .map(str::trim)
            .and_then(|value| value.parse::<IpAddr>().ok())
    {
        return ip.to_string();
    }

    request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(address)| address.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn is_json_response(response: &Response<Body>) -> bool {
    response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json"))
}

fn generic_error_for_status(status: StatusCode) -> Option<AppError> {
    match status {
        StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY => {
            Some(AppError::bad_request("请求格式无效").with_reason("REQUEST_INVALID"))
        }
        StatusCode::UNAUTHORIZED => {
            Some(AppError::unauthorized("请先登录").with_reason("AUTH_REQUIRED"))
        }
        StatusCode::FORBIDDEN => Some(AppError::forbidden("请求被拒绝")),
        StatusCode::NOT_FOUND => {
            Some(AppError::not_found("API 路径不存在").with_reason("API_ROUTE_NOT_FOUND"))
        }
        StatusCode::METHOD_NOT_ALLOWED => {
            Some(AppError::method_not_allowed("请求方法不支持").with_reason("METHOD_NOT_ALLOWED"))
        }
        StatusCode::PAYLOAD_TOO_LARGE => {
            Some(AppError::payload_too_large("请求内容过大").with_reason("REQUEST_BODY_TOO_LARGE"))
        }
        StatusCode::UNSUPPORTED_MEDIA_TYPE => Some(
            AppError::unsupported_media_type("请求内容类型不受支持")
                .with_reason("REQUEST_MEDIA_TYPE_UNSUPPORTED"),
        ),
        status if status.is_server_error() => Some(AppError::internal("服务内部错误")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{api_method_not_allowed, api_not_found, request_ip};
    use axum::{
        body::Body,
        extract::ConnectInfo,
        http::{Request, header::HeaderValue},
        response::IntoResponse,
    };
    use std::net::SocketAddr;

    #[test]
    fn proxy_header_is_ignored_by_default() {
        let request = request_with_ips("192.168.1.20:8080", "203.0.113.10");

        assert_eq!(request_ip(&request, false), "192.168.1.20");
    }

    #[test]
    fn trusted_proxy_header_uses_first_valid_ip() {
        let request = request_with_ips("192.168.1.20:8080", "203.0.113.10, 192.168.1.1");

        assert_eq!(request_ip(&request, true), "203.0.113.10");
    }

    #[test]
    fn invalid_proxy_header_falls_back_to_connection_ip() {
        let request = request_with_ips("192.168.1.20:8080", "not-an-ip");

        assert_eq!(request_ip(&request, true), "192.168.1.20");
    }

    #[tokio::test]
    async fn api_fallbacks_use_structured_errors() {
        let not_found = api_not_found().await.into_response();
        assert_eq!(not_found.status(), axum::http::StatusCode::NOT_FOUND);

        let method_not_allowed = api_method_not_allowed().await.into_response();
        assert_eq!(
            method_not_allowed.status(),
            axum::http::StatusCode::METHOD_NOT_ALLOWED
        );
    }

    fn request_with_ips(connection: &str, forwarded: &str) -> Request<Body> {
        let mut request = Request::new(Body::empty());
        request
            .headers_mut()
            .insert("x-forwarded-for", HeaderValue::from_str(forwarded).unwrap());
        request
            .extensions_mut()
            .insert(ConnectInfo(connection.parse::<SocketAddr>().unwrap()));
        request
    }
}
