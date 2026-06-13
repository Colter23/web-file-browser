mod audit;
mod auth;
mod file;
mod mapping;
mod ops;
mod search;
mod settings;
mod tasks;
mod trash;

use axum::{
    Router,
    extract::{ConnectInfo, Request, State},
    middleware,
    middleware::Next,
    response::Response,
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
    let protected_routes = Router::new()
        .merge(protected_auth_routes())
        .merge(audit_routes())
        .merge(file_routes())
        .merge(mapping_routes())
        .merge(settings_routes())
        .merge(task_routes())
        .merge(search_routes())
        .merge(trash_routes())
        .merge(ops_routes())
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth))
        .route_layer(middleware::from_fn_with_state(state, limit_ip));

    Router::new()
        .merge(public_auth_routes())
        .merge(public_ops_routes())
        .merge(protected_routes)
}

pub async fn api_index() -> &'static str {
    "Web File Browser Server"
}

pub async fn limit_ip(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = request_ip(&request, state.runtime_settings.trust_proxy_headers);
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

#[cfg(test)]
mod tests {
    use super::request_ip;
    use axum::{
        body::Body,
        extract::ConnectInfo,
        http::{Request, header::HeaderValue},
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
