use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, put},
};
use std::sync::Arc;

use crate::{
    app::AppState,
    error::AppError,
    models::{FolderNode, PathMapping},
};

pub fn mapping_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/mapping", get(list_mappings).post(create_mapping))
        .route("/mapping/root", get(mapping_root))
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
    Ok((StatusCode::CREATED, Json(id)))
}

async fn delete_mapping(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    state.mapping_store.delete(id).await?;
    Ok(StatusCode::OK)
}

async fn update_mapping(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(mapping): Json<PathMapping>,
) -> Result<StatusCode, AppError> {
    state.mapping_store.update(id, mapping).await?;
    Ok(StatusCode::OK)
}
