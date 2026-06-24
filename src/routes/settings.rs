use axum::{Json, Router, extract::State, routing::get};
use std::sync::Arc;

use crate::{app::AppState, models::RuntimeSettings};

pub fn settings_routes() -> Router<Arc<AppState>> {
    Router::new().route("/settings", get(get_settings))
}

async fn get_settings(State(state): State<Arc<AppState>>) -> Json<RuntimeSettings> {
    let mut settings = state.runtime_settings.clone();
    settings.auth_configured = state.auth_store.has_admin_password().await;
    Json(settings)
}
