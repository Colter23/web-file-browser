use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use serde::Serialize;
use std::sync::Arc;

use crate::{app::AppState, error::AppError, services::trash::TrashRecord};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TrashEmptyResponse {
    removed: usize,
}

pub fn trash_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/trash", get(list_trash))
        .route("/trash/empty", post(empty_trash))
        .route("/trash/{id}/restore", post(restore_trash))
        .route("/trash/{id}", delete(purge_trash))
}

async fn list_trash(State(state): State<Arc<AppState>>) -> Json<Vec<TrashRecord>> {
    Json(state.trash.list().await)
}

async fn restore_trash(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<TrashRecord>, AppError> {
    let record = state.trash.restore(id).await?;
    state
        .audit
        .record(
            "admin",
            "trash.restore",
            Some(&record.original_virtual_path),
            None,
        )
        .await?;
    Ok(Json(record))
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
