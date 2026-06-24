use axum::{
    Router,
    http::{
        HeaderName, HeaderValue, Method,
        header::{CONTENT_DISPOSITION, CONTENT_RANGE, ETAG},
    },
    routing::get,
};
use std::{path::Path, sync::Arc};
use tower_http::{
    cors::{AllowHeaders, AllowOrigin, CorsLayer},
    services::{ServeDir, ServeFile},
};

use crate::{
    config::AppConfig,
    error::AppError,
    models::RuntimeSettings,
    routes,
    services::{
        audit::AuditService, auth::AuthService, auth_store::AuthStore, mapping_store::MappingStore,
        request_limits::RequestLimits, search::SearchService, tasks::TaskService,
        trash::TrashService,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub mapping_store: MappingStore,
    pub auth_store: AuthStore,
    pub auth: AuthService,
    pub trash: TrashService,
    pub audit: AuditService,
    pub limits: RequestLimits,
    pub tasks: TaskService,
    pub search: SearchService,
    pub runtime_settings: RuntimeSettings,
}

pub async fn build(config: AppConfig) -> Result<Router, AppError> {
    let mapping_store = MappingStore::load(config.mapping_file.clone()).await?;
    let auth_store = AuthStore::load(config.auth_file.clone()).await?;
    let trash = TrashService::load(
        config.trash_dir.clone(),
        config.trash_retention_days,
        config.trash_max_bytes,
    )
    .await?;
    let audit = AuditService::load(
        config.audit_file.clone(),
        config.audit_max_bytes,
        config.audit_retention_files,
    )
    .await?;
    let limits = RequestLimits::new(
        config.max_dir_concurrency,
        config.max_transfer_concurrency,
        config.max_ip_concurrency,
    );
    let tasks = TaskService::new(
        config.max_task_concurrency,
        config.task_speed_limit_bytes_per_sec,
        config.task_history_limit,
    );
    let search = SearchService::new(config.index_enabled);
    let runtime_settings = RuntimeSettings {
        bind_address: config.bind_address.clone(),
        port: config.port,
        mapping_file: path_to_string(&config.mapping_file),
        config_file: path_to_string(&config.config_file),
        auth_file: path_to_string(&config.auth_file),
        trash_dir: path_to_string(&config.trash_dir),
        static_dir: path_to_string(&config.static_dir),
        cors_allowed_origins: config.cors_allowed_origins.clone(),
        trust_proxy_headers: config.trust_proxy_headers,
        max_edit_bytes: config.max_edit_bytes,
        editable_extensions: config.editable_extensions.clone(),
        editable_mime_types: config.editable_mime_types.clone(),
        max_upload_bytes: config.max_upload_bytes,
        max_dir_page_size: config.max_dir_page_size,
        max_dir_concurrency: config.max_dir_concurrency,
        max_transfer_concurrency: config.max_transfer_concurrency,
        max_ip_concurrency: config.max_ip_concurrency,
        max_task_concurrency: config.max_task_concurrency,
        task_history_limit: config.task_history_limit,
        task_speed_limit_bytes_per_sec: config.task_speed_limit_bytes_per_sec,
        max_extract_bytes: config.max_extract_bytes,
        max_extract_files: config.max_extract_files,
        max_extract_depth: config.max_extract_depth,
        index_enabled: config.index_enabled,
        index_rebuild_on_startup: config.index_rebuild_on_startup,
        index_scan_delay_ms: config.index_scan_delay_ms,
        audit_file: path_to_string(&config.audit_file),
        audit_max_bytes: config.audit_max_bytes,
        audit_retention_files: config.audit_retention_files,
        trash_retention_days: config.trash_retention_days,
        trash_max_bytes: config.trash_max_bytes,
        conflict_policy: config.conflict_policy,
        auth_configured: auth_store.has_admin_password().await,
    };

    let state = Arc::new(AppState {
        mapping_store,
        auth_store,
        auth: AuthService::default(),
        trash,
        audit,
        limits,
        tasks,
        search,
        runtime_settings,
    });

    if config.index_enabled && config.index_rebuild_on_startup {
        let search = state.search.clone();
        let snapshot = state.mapping_store.snapshot().await;
        let scan_delay_ms = config.index_scan_delay_ms;
        tokio::spawn(async move {
            if let Err(error) = search.rebuild(snapshot, scan_delay_ms).await {
                tracing::warn!("搜索索引初始化失败: {error}");
            }
        });
    } else if config.index_enabled {
        tracing::info!("搜索索引已启用，但启动时不会自动扫描挂载目录");
    }

    let index_file = config.static_dir.join("index.html");
    let static_files = ServeDir::new(config.static_dir).fallback(ServeFile::new(index_file));
    let cors = build_cors_layer(&config.cors_allowed_origins)?;

    Ok(Router::new()
        .route("/api", get(routes::api_index))
        .route("/api/", get(routes::api_index))
        .nest("/api", routes::api_routes(state.clone()))
        .fallback_service(static_files)
        .layer(cors)
        .with_state(state))
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn build_cors_layer(allowed_origins: &[String]) -> Result<CorsLayer, AppError> {
    let origins = allowed_origins
        .iter()
        .map(|origin| {
            if origin == "*" {
                return Err(AppError::bad_request(
                    "WEB_FILE_BROWSER_CORS_ORIGINS 不支持 *，请显式填写可信来源",
                ));
            }
            HeaderValue::from_str(origin).map_err(|error| {
                AppError::bad_request(format!("CORS 来源配置无效: {origin} ({error})"))
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;

    Ok(CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_credentials(true)
        .allow_methods([
            Method::GET,
            Method::HEAD,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(AllowHeaders::mirror_request())
        .expose_headers([
            ETAG,
            CONTENT_RANGE,
            CONTENT_DISPOSITION,
            HeaderName::from_static("accept-ranges"),
        ]))
}

#[cfg(test)]
mod tests;
