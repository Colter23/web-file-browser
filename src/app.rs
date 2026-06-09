use axum::{Router, http::Method, routing::get};
use std::{env, path::Path, sync::Arc};
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
        audit::AuditService, auth::AuthService, mapping_store::MappingStore,
        request_limits::RequestLimits, search::SearchService, settings::SettingsStore,
        tasks::TaskService, trash::TrashService,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub mapping_store: MappingStore,
    pub settings: SettingsStore,
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
    let settings = SettingsStore::load(
        config.config_file.clone(),
        env::var("WEB_FILE_BROWSER_ADMIN_PASSWORD").ok(),
    )
    .await?;
    let trash = TrashService::load(
        config.trash_dir.clone(),
        config.trash_retention_days,
        config.trash_max_bytes,
    )
    .await?;
    let audit = AuditService::load(config.audit_file.clone()).await?;
    let limits = RequestLimits::new(
        config.max_dir_concurrency,
        config.max_transfer_concurrency,
        config.max_ip_concurrency,
    );
    let tasks = TaskService::new(
        config.max_task_concurrency,
        config.task_speed_limit_bytes_per_sec,
    );
    let search = SearchService::new(config.index_enabled);
    let runtime_settings = RuntimeSettings {
        bind_address: config.bind_address.clone(),
        port: config.port,
        mapping_file: path_to_string(&config.mapping_file),
        config_file: path_to_string(&config.config_file),
        trash_dir: path_to_string(&config.trash_dir),
        static_dir: path_to_string(&config.static_dir),
        max_upload_bytes: config.max_upload_bytes,
        max_dir_page_size: config.max_dir_page_size,
        max_dir_concurrency: config.max_dir_concurrency,
        max_transfer_concurrency: config.max_transfer_concurrency,
        max_ip_concurrency: config.max_ip_concurrency,
        max_task_concurrency: config.max_task_concurrency,
        task_speed_limit_bytes_per_sec: config.task_speed_limit_bytes_per_sec,
        index_enabled: config.index_enabled,
        index_scan_delay_ms: config.index_scan_delay_ms,
        audit_file: path_to_string(&config.audit_file),
        trash_retention_days: config.trash_retention_days,
        trash_max_bytes: config.trash_max_bytes,
        auth_configured: settings.has_admin_password().await,
    };

    let state = Arc::new(AppState {
        mapping_store,
        settings,
        auth: AuthService::default(),
        trash,
        audit,
        limits,
        tasks,
        search,
        runtime_settings,
    });

    if config.index_enabled {
        let search = state.search.clone();
        let snapshot = state.mapping_store.snapshot().await;
        let scan_delay_ms = config.index_scan_delay_ms;
        tokio::spawn(async move {
            if let Err(error) = search.rebuild(snapshot, scan_delay_ms).await {
                tracing::warn!("搜索索引初始化失败: {error}");
            }
        });
    }

    let index_file = config.static_dir.join("index.html");
    let static_files = ServeDir::new(config.static_dir).fallback(ServeFile::new(index_file));

    Ok(Router::new()
        .route("/api", get(routes::api_index))
        .route("/api/", get(routes::api_index))
        .nest("/api", routes::api_routes(state.clone()))
        .fallback_service(static_files)
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::predicate(|_, _| true))
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
                .allow_headers(AllowHeaders::mirror_request()),
        )
        .with_state(state))
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}
