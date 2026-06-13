use axum::{Json, Router, extract::State, routing::post};
use serde::Serialize;
use std::sync::Arc;

use crate::{app::AppState, error::AppError};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AuditCleanupResponse {
    removed: usize,
}

pub fn audit_routes() -> Router<Arc<AppState>> {
    Router::new().route("/audit/cleanup", post(cleanup_audit))
}

async fn cleanup_audit(
    State(state): State<Arc<AppState>>,
) -> Result<Json<AuditCleanupResponse>, AppError> {
    state
        .audit
        .record("admin", "audit.cleanup", None, None)
        .await?;
    let removed = state.audit.cleanup_rotated().await?;
    Ok(Json(AuditCleanupResponse { removed }))
}
