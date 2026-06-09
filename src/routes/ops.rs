use axum::{Json, Router, extract::State, routing::get};
use std::sync::Arc;

use crate::{
    app::AppState,
    models::{HealthResponse, MetricsResponse},
};

pub fn ops_routes() -> Router<Arc<AppState>> {
    Router::new().route("/metrics", get(metrics))
}

pub fn public_ops_routes() -> Router<Arc<AppState>> {
    Router::new().route("/health", get(health))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn metrics(State(state): State<Arc<AppState>>) -> Json<MetricsResponse> {
    Json(MetricsResponse {
        mappings: state.mapping_store.list().await.len(),
        active_sessions: state.auth.count().await,
        tasks_total: state.tasks.count_total().await,
        tasks_running: state.tasks.count_running().await,
        trash_entries: state.trash.count().await,
        indexed_entries: state.search.count().await,
    })
}
