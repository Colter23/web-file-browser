use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    app::AppState,
    error::AppError,
    models::ConflictPolicy,
    services::{
        path_resolver::MappingSnapshot,
        trash::{TrashRecord, TrashRestoreResult},
    },
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TrashEmptyResponse {
    removed: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TrashCleanupResponse {
    removed: usize,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TrashWriteQuery {
    #[serde(default, alias = "conflict")]
    conflict_policy: Option<String>,
}

impl TrashWriteQuery {
    fn parse_conflict_policy(&self) -> Result<Option<ConflictPolicy>, AppError> {
        self.conflict_policy
            .as_deref()
            .map(|value| {
                value
                    .parse()
                    .map_err(|_| AppError::bad_request(format!("不支持的 conflictPolicy: {value}")))
            })
            .transpose()
    }
}

pub fn trash_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/trash", get(list_trash))
        .route("/trash/cleanup", post(cleanup_trash))
        .route("/trash/empty", post(empty_trash))
        .route("/trash/{id}/restore", post(restore_trash))
        .route("/trash/{id}", delete(purge_trash))
}

async fn list_trash(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<TrashRecord>>, AppError> {
    state.trash.cleanup_retention_if_due().await?;
    Ok(Json(state.trash.list().await))
}

async fn cleanup_trash(
    State(state): State<Arc<AppState>>,
) -> Result<Json<TrashCleanupResponse>, AppError> {
    let removed = state.trash.cleanup_retention().await?;
    let detail = format!("removed={removed}");
    state
        .audit
        .record("admin", "trash.cleanup", None, Some(&detail))
        .await?;
    Ok(Json(TrashCleanupResponse { removed }))
}

async fn restore_trash(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(query): Query<TrashWriteQuery>,
) -> Result<Json<TrashRestoreResult>, AppError> {
    let policy = query
        .parse_conflict_policy()?
        .unwrap_or(state.runtime_settings.conflict_policy);
    let snapshot = state.mapping_store.snapshot().await;
    let restored = state.trash.restore(snapshot.clone(), id, policy).await?;
    index_upsert_ignore(&state, snapshot, &restored.restored_virtual_path).await;
    state
        .audit
        .record(
            "admin",
            "trash.restore",
            Some(&restored.restored_virtual_path),
            None,
        )
        .await?;
    Ok(Json(restored))
}

async fn index_upsert_ignore(state: &AppState, snapshot: Arc<MappingSnapshot>, virtual_path: &str) {
    if let Err(error) = state
        .search
        .upsert_virtual_path(snapshot, virtual_path.to_string())
        .await
    {
        tracing::warn!("更新搜索索引失败: {error}");
    }
}

async fn purge_trash(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    state.trash.purge(id.clone()).await?;
    state
        .audit
        .record("admin", "trash.purge", None, Some(&id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn empty_trash(
    State(state): State<Arc<AppState>>,
) -> Result<Json<TrashEmptyResponse>, AppError> {
    let removed = state.trash.empty().await?;
    let detail = format!("removed={removed}");
    state
        .audit
        .record("admin", "trash.empty", None, Some(&detail))
        .await?;
    Ok(Json(TrashEmptyResponse { removed }))
}
