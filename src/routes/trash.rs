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
        trash::{TrashBatchItemError, TrashRecord, TrashRestoreResult},
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TrashBatchRequest {
    ids: Vec<String>,
    #[serde(default, alias = "conflict")]
    conflict_policy: Option<ConflictPolicy>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TrashBatchRestoreResponse {
    restored: Vec<TrashRestoreResult>,
    errors: Vec<TrashBatchItemError>,
    success: usize,
    failed: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TrashBatchPurgeResponse {
    purged: Vec<String>,
    errors: Vec<TrashBatchItemError>,
    success: usize,
    failed: usize,
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
        .route("/trash/batch/restore", post(restore_trash_batch))
        .route("/trash/batch/purge", post(purge_trash_batch))
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

async fn restore_trash_batch(
    State(state): State<Arc<AppState>>,
    Json(request): Json<TrashBatchRequest>,
) -> Result<Json<TrashBatchRestoreResponse>, AppError> {
    let ids = validate_batch_ids(request.ids)?;
    let policy = request
        .conflict_policy
        .unwrap_or(state.runtime_settings.conflict_policy);
    let snapshot = state.mapping_store.snapshot().await;
    let outcome = state
        .trash
        .restore_batch(snapshot.clone(), ids, policy)
        .await?;

    for result in &outcome.restored {
        index_upsert_ignore(&state, snapshot.clone(), &result.restored_virtual_path).await;
        audit_ignore(
            &state,
            "trash.restore",
            Some(&result.restored_virtual_path),
            None,
        )
        .await;
    }

    let success = outcome.restored.len();
    let failed = outcome.errors.len();
    Ok(Json(TrashBatchRestoreResponse {
        restored: outcome.restored,
        errors: outcome.errors,
        success,
        failed,
    }))
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

async fn audit_ignore(state: &AppState, action: &str, target: Option<&str>, detail: Option<&str>) {
    if let Err(error) = state.audit.record("admin", action, target, detail).await {
        tracing::warn!("写入审计日志失败: {error}");
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

async fn purge_trash_batch(
    State(state): State<Arc<AppState>>,
    Json(request): Json<TrashBatchRequest>,
) -> Result<Json<TrashBatchPurgeResponse>, AppError> {
    let ids = validate_batch_ids(request.ids)?;
    let outcome = state.trash.purge_batch(ids).await?;

    for id in &outcome.purged {
        audit_ignore(&state, "trash.purge", None, Some(id)).await;
    }

    let success = outcome.purged.len();
    let failed = outcome.errors.len();
    Ok(Json(TrashBatchPurgeResponse {
        purged: outcome.purged,
        errors: outcome.errors,
        success,
        failed,
    }))
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

fn validate_batch_ids(ids: Vec<String>) -> Result<Vec<String>, AppError> {
    if ids.is_empty() {
        return Err(AppError::bad_request("批量回收站操作 ids 不能为空"));
    }
    let mut clean_ids = Vec::with_capacity(ids.len());
    for id in ids {
        let id = id.trim();
        if id.is_empty() {
            return Err(AppError::bad_request("批量回收站操作 ids 包含空编号"));
        }
        clean_ids.push(id.to_string());
    }
    Ok(clean_ids)
}
