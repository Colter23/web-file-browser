use axum::{
    Router,
    http::{
        HeaderName, HeaderValue, Method,
        header::{CONTENT_DISPOSITION, CONTENT_RANGE, ETAG},
    },
    routing::get,
};
use std::sync::Arc;
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
        audit::AuditService, auth::AuthService, auth_store::AuthStore, favorites::FavoriteService,
        mapping_store::MappingStore, request_limits::RequestLimits, search::SearchService,
        settings::SettingsService, tasks::TaskService, trash::TrashService,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub mapping_store: MappingStore,
    pub auth_store: AuthStore,
    pub auth: AuthService,
    pub favorites: FavoriteService,
    pub trash: TrashService,
    pub audit: AuditService,
    pub limits: RequestLimits,
    pub tasks: TaskService,
    pub search: SearchService,
    pub settings: SettingsService,
}

pub async fn build(config: AppConfig) -> Result<Router, AppError> {
    let mapping_store = MappingStore::load(config.mapping_file.clone()).await?;
    let auth_store = AuthStore::load(config.auth_file.clone()).await?;
    let favorites = FavoriteService::load(config.favorites_file.clone()).await?;
    let trash = TrashService::load(
        config.trash_dir.clone(),
        config.trash_retention_days,
        config.trash_max_bytes,
    )
    .await?;
    let audit = AuditService::load(
        config.audit_file.clone(),
        config.audit_enabled,
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
    let settings = SettingsService::new(&config);

    let state = Arc::new(AppState {
        mapping_store,
        auth_store,
        auth: AuthService::default(),
        favorites,
        trash,
        audit,
        limits,
        tasks,
        search,
        settings,
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

impl AppState {
    pub async fn apply_runtime_settings(&self, settings: &RuntimeSettings) {
        self.limits.update(
            settings.max_dir_concurrency,
            settings.max_transfer_concurrency,
            settings.max_ip_concurrency,
        );
        self.tasks.update(
            settings.max_task_concurrency,
            settings.task_speed_limit_bytes_per_sec,
            settings.task_history_limit,
        );
        self.search.set_enabled(settings.index_enabled).await;
        self.trash
            .update_policy(settings.trash_retention_days, settings.trash_max_bytes);
        self.audit.update_policy(
            settings.audit_enabled,
            settings.audit_max_bytes,
            settings.audit_retention_files,
        );
    }
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
