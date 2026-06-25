use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use std::sync::Arc;

use crate::{
    app::AppState,
    error::AppError,
    models::{SettingsResponse, UpdateSettingsRequest},
};

pub fn settings_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/settings", get(get_settings).patch(update_settings))
        .route("/settings/reload", post(reload_settings))
}

async fn get_settings(State(state): State<Arc<AppState>>) -> Json<SettingsResponse> {
    Json(settings_response(&state).await)
}

async fn update_settings(
    State(state): State<Arc<AppState>>,
    Json(request): Json<UpdateSettingsRequest>,
) -> Result<Json<SettingsResponse>, AppError> {
    if request.startup.is_some() {
        return Err(AppError::bad_request(
            "启动配置不支持在线修改，请编辑配置文件后重启服务",
        ));
    }
    if let Some(runtime) = request.runtime {
        let runtime = state.settings.patch_runtime(runtime).await?;
        state.apply_runtime_settings(&runtime).await;
    }
    Ok(Json(settings_response(&state).await))
}

async fn reload_settings(
    State(state): State<Arc<AppState>>,
) -> Result<Json<SettingsResponse>, AppError> {
    let runtime = state.settings.reload_runtime().await?;
    state.apply_runtime_settings(&runtime).await;
    Ok(Json(settings_response(&state).await))
}

async fn settings_response(state: &AppState) -> SettingsResponse {
    let auth_configured = state.auth_store.has_admin_password().await;
    state.settings.response(auth_configured).await
}
