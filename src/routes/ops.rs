use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    app::AppState,
    models::{HealthResponse, MetricsResponse, ReadinessCheck, ReadinessResponse, RuntimeSettings},
};

pub fn ops_routes() -> Router<Arc<AppState>> {
    Router::new().route("/metrics", get(metrics))
}

pub fn public_ops_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(readiness))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn metrics(State(state): State<Arc<AppState>>) -> Json<MetricsResponse> {
    Json(MetricsResponse {
        mappings: state.mapping_store.count().await,
        active_sessions: state.auth.count().await,
        trash_entries: state.trash.count().await,
        tasks: state.tasks.metrics().await,
        limits: state.limits.metrics().await,
        index: state.search.status().await,
    })
}

async fn readiness(State(state): State<Arc<AppState>>) -> (StatusCode, Json<ReadinessResponse>) {
    let checks = readiness_checks(&state).await;
    let ready = checks.iter().all(|check| check.status == "ok");
    let status = if ready { "ok" } else { "notReady" }.to_string();
    let status_code = if ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(ReadinessResponse {
            status,
            version: env!("CARGO_PKG_VERSION").to_string(),
            checks,
        }),
    )
}

async fn readiness_checks(state: &AppState) -> Vec<ReadinessCheck> {
    let mut checks = Vec::new();

    checks.push(if state.auth_store.has_admin_password().await {
        readiness_ok("auth", "管理员密码已初始化")
    } else {
        readiness_ok("auth", "管理员密码尚未初始化，等待首次进入 Web 界面设置")
    });
    let settings = state.runtime_settings.clone();
    match tokio::task::spawn_blocking(move || readiness_file_system_checks(&settings)).await {
        Ok(mut file_system_checks) => checks.append(&mut file_system_checks),
        Err(error) => checks.push(readiness_error(
            "fileSystem",
            format!("就绪检查执行失败: {error}"),
        )),
    }
    checks
}

fn readiness_file_system_checks(settings: &RuntimeSettings) -> Vec<ReadinessCheck> {
    vec![
        check_file_parent_writable("configStore", &settings.config_file),
        check_file_parent_writable("authStore", &settings.auth_file),
        check_file_parent_writable("favoritesStore", &settings.favorites_file),
        check_file_parent_writable("mappingStore", &settings.mapping_file),
        check_directory_writable("trash", &settings.trash_dir),
        check_file_parent_writable("audit", &settings.audit_file),
        check_directory_readable("staticFiles", &settings.static_dir),
    ]
}

fn check_file_parent_writable(name: &str, file_path: &str) -> ReadinessCheck {
    let path = Path::new(file_path);
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    check_directory_writable(name, parent)
}

fn check_directory_writable(name: &str, dir: impl AsRef<Path>) -> ReadinessCheck {
    let dir = dir.as_ref();
    if let Err(error) = std::fs::create_dir_all(dir) {
        return readiness_error(name, format!("目录不可创建: {error}"));
    }

    let probe_path = dir.join(format!(".wfb-ready-{}.tmp", uuid::Uuid::new_v4()));
    match std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&probe_path)
    {
        Ok(_) => {
            let _ = std::fs::remove_file(&probe_path);
            readiness_ok(name, "目录可写")
        }
        Err(error) => readiness_error(name, format!("目录不可写: {error}")),
    }
}

fn check_directory_readable(name: &str, dir: impl AsRef<Path>) -> ReadinessCheck {
    let dir = PathBuf::from(dir.as_ref());
    match std::fs::metadata(&dir) {
        Ok(metadata) if metadata.is_dir() => readiness_ok(name, "目录可读"),
        Ok(_) => readiness_error(name, "路径不是目录"),
        Err(error) => readiness_error(name, format!("目录不可读: {error}")),
    }
}

fn readiness_ok(name: &str, message: impl Into<String>) -> ReadinessCheck {
    ReadinessCheck {
        name: name.to_string(),
        status: "ok".to_string(),
        message: message.into(),
    }
}

fn readiness_error(name: &str, message: impl Into<String>) -> ReadinessCheck {
    ReadinessCheck {
        name: name.to_string(),
        status: "error".to_string(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::{check_directory_readable, check_directory_writable};

    #[test]
    fn readiness_writable_check_creates_and_removes_probe() {
        let dir = temp_dir("web-file-browser-ready-writable-test");
        std::fs::create_dir_all(&dir).unwrap();

        let check = check_directory_writable("data", &dir);

        assert_eq!(check.status, "ok");
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn readiness_readable_check_rejects_file() {
        let dir = temp_dir("web-file-browser-ready-readable-test");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("not-dir");
        std::fs::write(&file, "hello").unwrap();

        let check = check_directory_readable("staticFiles", &file);

        assert_eq!(check.status, "error");
        std::fs::remove_dir_all(dir).unwrap();
    }

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nonce}"))
    }
}
