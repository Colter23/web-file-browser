use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put},
};
use std::sync::Arc;

use crate::{
    app::AppState,
    error::AppError,
    models::{FolderNode, PathMapping, ReorderMappingsRequest},
};

pub fn mapping_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/mapping", get(list_mappings).post(create_mapping))
        .route("/mapping/root", get(mapping_root))
        .route("/mapping/reorder", post(reorder_mappings))
        .route("/mapping/{id}", put(update_mapping).delete(delete_mapping))
}

async fn list_mappings(State(state): State<Arc<AppState>>) -> Json<Vec<PathMapping>> {
    Json(state.mapping_store.list().await)
}

async fn mapping_root(State(state): State<Arc<AppState>>) -> Json<Option<FolderNode>> {
    Json(state.mapping_store.root_node().await)
}

async fn create_mapping(
    State(state): State<Arc<AppState>>,
    Json(mapping): Json<PathMapping>,
) -> Result<(StatusCode, Json<i64>), AppError> {
    let id = state.mapping_store.create(mapping).await?;
    if let Some(mapping) = state.mapping_store.get(id).await {
        index_remove_mount_ignore(&state, &mapping.mount_path).await;
    }
    Ok((StatusCode::CREATED, Json(id)))
}

async fn delete_mapping(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let old_mapping = state.mapping_store.get(id).await;
    state.mapping_store.delete(id).await?;
    if let Some(mapping) = old_mapping {
        index_remove_mount_ignore(&state, &mapping.mount_path).await;
    }
    Ok(StatusCode::OK)
}

async fn update_mapping(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(mapping): Json<PathMapping>,
) -> Result<StatusCode, AppError> {
    let old_mapping = state.mapping_store.get(id).await;
    state.mapping_store.update(id, mapping).await?;
    if let Some(mapping) = old_mapping {
        index_remove_mount_ignore(&state, &mapping.mount_path).await;
    }
    if let Some(mapping) = state.mapping_store.get(id).await {
        index_remove_mount_ignore(&state, &mapping.mount_path).await;
    }
    Ok(StatusCode::OK)
}

async fn reorder_mappings(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ReorderMappingsRequest>,
) -> Result<StatusCode, AppError> {
    state.mapping_store.reorder(request).await?;
    Ok(StatusCode::OK)
}

async fn index_remove_mount_ignore(state: &AppState, mount_path: &str) {
    if let Err(error) = state.search.remove_mount(mount_path).await {
        tracing::warn!("清理挂载搜索索引失败: {error}");
    }
}
