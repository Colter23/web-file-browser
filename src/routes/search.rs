use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    app::AppState,
    error::AppError,
    models::{IndexStatus, SearchResponse, SearchResult},
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchQuery {
    q: Option<String>,
    mount: Option<String>,
    #[serde(rename = "type")]
    entry_type: Option<String>,
    offset: Option<usize>,
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct RecentQuery {
    limit: Option<usize>,
}

pub fn search_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/search", get(search))
        .route("/recent", get(recent))
        .route("/index/rebuild", post(rebuild_index))
        .route("/index/cancel", post(cancel_index_rebuild))
        .route("/index/status", get(index_status))
}

async fn search(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, AppError> {
    let limit = bounded_limit(query.limit, state.runtime_settings.max_dir_page_size)?;
    Ok(Json(
        state
            .search
            .search(
                query.q,
                query.mount,
                query.entry_type,
                query.offset.unwrap_or(0),
                limit,
            )
            .await,
    ))
}

async fn recent(
    State(state): State<Arc<AppState>>,
    Query(query): Query<RecentQuery>,
) -> Result<Json<Vec<SearchResult>>, AppError> {
    let limit = bounded_limit(
        query.limit.or(Some(50)),
        state.runtime_settings.max_dir_page_size,
    )?;
    Ok(Json(state.search.recent(limit).await))
}

async fn rebuild_index(State(state): State<Arc<AppState>>) -> Result<StatusCode, AppError> {
    if !state.runtime_settings.index_enabled {
        return Err(AppError::bad_request("搜索索引未启用"));
    }
    if state.search.status().await.state == "scanning" {
        return Err(AppError::conflict("索引正在重建"));
    }
    let snapshot = state.mapping_store.snapshot().await;
    let search = state.search.clone();
    let scan_delay_ms = state.runtime_settings.index_scan_delay_ms;
    tokio::spawn(async move {
        if let Err(error) = search.rebuild(snapshot, scan_delay_ms).await {
            tracing::warn!("搜索索引重建失败: {error}");
        }
    });
    Ok(StatusCode::ACCEPTED)
}

async fn index_status(State(state): State<Arc<AppState>>) -> Json<IndexStatus> {
    Json(state.search.status().await)
}

async fn cancel_index_rebuild(State(state): State<Arc<AppState>>) -> Result<StatusCode, AppError> {
    state.search.cancel_rebuild().await?;
    Ok(StatusCode::ACCEPTED)
}

fn bounded_limit(limit: Option<usize>, max_limit: usize) -> Result<usize, AppError> {
    let limit = limit.unwrap_or(max_limit);
    if limit == 0 {
        return Err(AppError::bad_request("分页大小必须大于 0"));
    }
    Ok(limit.min(max_limit))
}
