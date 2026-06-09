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
use std::{net::SocketAddr, sync::Arc};

use crate::{
    app::AppState,
    error::AppError,
    routes::{
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
    let ip = request_ip(&request);
    let _permit = state.limits.acquire_ip(ip).await?;
    Ok(next.run(request).await)
}

fn request_ip(request: &Request) -> String {
    request
        .headers()
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .or_else(|| {
            request
                .extensions()
                .get::<ConnectInfo<SocketAddr>>()
                .map(|ConnectInfo(address)| address.ip().to_string())
        })
        .unwrap_or_else(|| "unknown".to_string())
}
