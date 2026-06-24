use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    app::AppState,
    error::AppError,
    models::{
        CreateFavoriteRequest, FavoriteResponse, ReorderFavoritesRequest, UpdateFavoriteRequest,
    },
};

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FavoriteListQuery {
    #[serde(default)]
    check: bool,
}

pub fn favorite_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/favorites", get(list_favorites).post(create_favorite))
        .route("/favorites/reorder", post(reorder_favorites))
        .route(
            "/favorites/{id}",
            axum::routing::patch(update_favorite).delete(delete_favorite),
        )
}

async fn list_favorites(
    State(state): State<Arc<AppState>>,
    Query(query): Query<FavoriteListQuery>,
) -> Result<Json<Vec<FavoriteResponse>>, AppError> {
    let snapshot = state.mapping_store.snapshot().await;
    Ok(Json(state.favorites.list(snapshot, query.check).await?))
}

async fn create_favorite(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateFavoriteRequest>,
) -> Result<(StatusCode, Json<FavoriteResponse>), AppError> {
    let snapshot = state.mapping_store.snapshot().await;
    let response = state.favorites.create(snapshot, request).await?;
    audit_ignore(&state, "favorite.create", Some(&response.path), None).await;
    Ok((StatusCode::CREATED, Json(response)))
}

async fn update_favorite(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(request): Json<UpdateFavoriteRequest>,
) -> Result<Json<FavoriteResponse>, AppError> {
    let snapshot = state.mapping_store.snapshot().await;
    let response = state.favorites.update(snapshot, id, request).await?;
    audit_ignore(&state, "favorite.update", Some(&response.path), None).await;
    Ok(Json(response))
}

async fn delete_favorite(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    state.favorites.delete(id.clone()).await?;
    audit_ignore(&state, "favorite.delete", None, Some(&id)).await;
    Ok(StatusCode::NO_CONTENT)
}

async fn reorder_favorites(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ReorderFavoritesRequest>,
) -> Result<StatusCode, AppError> {
    state.favorites.reorder(request).await?;
    audit_ignore(&state, "favorite.reorder", None, None).await;
    Ok(StatusCode::OK)
}

async fn audit_ignore(state: &AppState, action: &str, path: Option<&str>, detail: Option<&str>) {
    if let Err(error) = state.audit.record("admin", action, path, detail).await {
        tracing::warn!("写入审计日志失败: {error}");
    }
}
