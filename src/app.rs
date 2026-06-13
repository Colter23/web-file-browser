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
        config.initial_admin_password.clone(),
    )
    .await?;
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
mod tests {
    use axum::{
        Router,
        body::{Body, to_bytes},
        extract::ConnectInfo,
        http::{
            Method, Request, Response, StatusCode,
            header::{
                CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, COOKIE, ETAG,
                IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED, RANGE, SET_COOKIE,
            },
        },
    };
    use bytes::Bytes;
    use futures_util::stream;
    use serde_json::{Value, json};
    use std::{
        fs,
        io::Write,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };
    use tower::ServiceExt;
    use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

    use crate::{
        app::{build, build_cors_layer},
        config::AppConfig,
        error::AppError,
        models::ConflictPolicy,
    };

    #[test]
    fn cors_rejects_wildcard_origin() {
        let error = build_cors_layer(&["*".to_string()]).unwrap_err();

        assert!(matches!(error, AppError::BadRequest(_)));
    }

    #[test]
    fn cors_accepts_empty_origin_list() {
        assert!(build_cors_layer(&[]).is_ok());
    }

    #[tokio::test]
    async fn health_endpoint_is_public() {
        let (_root, app) = test_app("health-public").await;

        let response = app
            .oneshot(empty_request(Method::GET, "/api/health"))
            .await
            .unwrap();
        let status = response.status();
        let body = json_body(response).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "ok");
    }

    #[tokio::test]
    async fn protected_api_requires_login_with_json_error() {
        let (_root, app) = test_app("protected-api").await;

        let response = app
            .oneshot(empty_request(Method::GET, "/api/file"))
            .await
            .unwrap();
        let status = response.status();
        let body = json_body(response).await;

        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["code"], "UNAUTHORIZED");
        assert_eq!(body["message"], "请先登录");
    }

    #[tokio::test]
    async fn readonly_mount_rejects_write_apis_with_json_error() {
        let (root, app) = test_app("readonly-api").await;
        let files_dir = root.path().join("readonly");
        tokio::fs::create_dir_all(&files_dir).await.unwrap();
        tokio::fs::write(files_dir.join("hello.txt"), b"readonly-body")
            .await
            .unwrap();

        let cookie = login_cookie(&app).await;
        let mapping = json!({
            "mountPath": "/readonly",
            "folderPath": path_text(&files_dir),
            "remark": "只读挂载",
            "order": 0,
            "writable": false
        });
        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/mapping",
                Some(&cookie),
                mapping,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let metadata = get_json(&app, &cookie, "/api/file/readonly").await;
        assert_eq!(metadata["path"], "/readonly");

        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/file/readonly",
                Some(&cookie),
                json!({ "type": "file", "name": "blocked.txt" }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let body = json_body(response).await;
        assert_eq!(body["code"], "FORBIDDEN");
        assert_eq!(body["message"], "挂载点是只读模式");
        assert!(!files_dir.join("blocked.txt").exists());

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::PUT)
                    .uri("/api/content/readonly/hello.txt")
                    .header(COOKIE, &cookie)
                    .header("If-Match", "*")
                    .body(Body::from("blocked-save"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let body = json_body(response).await;
        assert_eq!(body["code"], "FORBIDDEN");
        assert_eq!(
            tokio::fs::read_to_string(files_dir.join("hello.txt"))
                .await
                .unwrap(),
            "readonly-body"
        );
    }

    #[tokio::test]
    async fn api_rejects_parent_segments_with_json_error() {
        let (root, app) = test_app("path-traversal-api").await;
        let files_dir = root.path().join("files");
        tokio::fs::create_dir_all(&files_dir).await.unwrap();

        let cookie = login_cookie(&app).await;
        let mapping = json!({
            "mountPath": "/docs",
            "folderPath": path_text(&files_dir),
            "remark": "测试挂载",
            "order": 0,
            "writable": true
        });
        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/mapping",
                Some(&cookie),
                mapping,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::GET,
                "/api/file/docs/%2E%2E/secret",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = json_body(response).await;
        assert_eq!(body["code"], "BAD_REQUEST");

        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/file/docs",
                Some(&cookie),
                json!({ "type": "file", "name": "../evil.txt" }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = json_body(response).await;
        assert_eq!(body["code"], "BAD_REQUEST");
        assert!(!root.path().join("evil.txt").exists());
        assert!(!files_dir.join("evil.txt").exists());
    }

    #[tokio::test]
    async fn cross_mount_move_is_rejected_through_api() {
        let (root, app) = test_app("cross-mount-move-api").await;
        let left_dir = root.path().join("left");
        let right_dir = root.path().join("right");
        tokio::fs::create_dir_all(&left_dir).await.unwrap();
        tokio::fs::create_dir_all(&right_dir).await.unwrap();
        tokio::fs::write(left_dir.join("hello.txt"), b"left-body")
            .await
            .unwrap();

        let cookie = login_cookie(&app).await;
        create_mapping(&app, &cookie, "/left", &left_dir, true).await;
        create_mapping(&app, &cookie, "/right", &right_dir, true).await;

        let response = app
            .clone()
            .oneshot(json_request(
                Method::PATCH,
                "/api/file/left/hello.txt",
                Some(&cookie),
                json!({ "targetPath": "/right/hello.txt" }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = json_body(response).await;
        assert_eq!(body["code"], "BAD_REQUEST");
        assert_eq!(body["message"], "不能跨挂载点移动文件");
        assert!(left_dir.join("hello.txt").is_file());
        assert!(!right_dir.join("hello.txt").exists());
    }

    #[tokio::test]
    async fn copy_task_reports_readonly_target_error() {
        let (root, app) = test_app("copy-readonly-target-api").await;
        let source_dir = root.path().join("source");
        let readonly_dir = root.path().join("readonly-target");
        tokio::fs::create_dir_all(&source_dir).await.unwrap();
        tokio::fs::create_dir_all(&readonly_dir).await.unwrap();
        tokio::fs::write(source_dir.join("hello.txt"), b"copy-body")
            .await
            .unwrap();

        let cookie = login_cookie(&app).await;
        create_mapping(&app, &cookie, "/source", &source_dir, true).await;
        create_mapping(&app, &cookie, "/readonly", &readonly_dir, false).await;

        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/tasks/copy",
                Some(&cookie),
                json!({
                    "sources": ["/source/hello.txt"],
                    "targetPath": "/readonly",
                    "conflictPolicy": "reject"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let task_id = json_body(response).await["id"]
            .as_str()
            .unwrap()
            .to_string();

        let task = wait_task_terminal(&app, &cookie, &task_id).await;
        assert_eq!(task["state"], "failed");
        assert_eq!(task["errors"].as_array().unwrap().len(), 1);
        assert_eq!(task["errors"][0]["path"], "/source/hello.txt");
        assert!(
            task["errors"][0]["message"]
                .as_str()
                .unwrap()
                .contains("挂载点是只读模式")
        );
        assert!(!readonly_dir.join("hello.txt").exists());
    }

    #[tokio::test]
    async fn copy_task_rejects_symlink_source_through_api_when_available() {
        let (root, app) = test_app("copy-symlink-source-api").await;
        let source_dir = root.path().join("source");
        let target_dir = root.path().join("target");
        tokio::fs::create_dir_all(&source_dir).await.unwrap();
        tokio::fs::create_dir_all(&target_dir).await.unwrap();
        let source = source_dir.join("hello.txt");
        let link = source_dir.join("hello-link.txt");
        tokio::fs::write(&source, b"copy-body").await.unwrap();
        if !try_create_file_symlink(&source, &link) {
            return;
        }

        let cookie = login_cookie(&app).await;
        create_mapping(&app, &cookie, "/source", &source_dir, true).await;
        create_mapping(&app, &cookie, "/target", &target_dir, true).await;

        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/tasks/copy",
                Some(&cookie),
                json!({
                    "sources": ["/source/hello-link.txt"],
                    "targetPath": "/target",
                    "conflictPolicy": "reject"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let task_id = json_body(response).await["id"]
            .as_str()
            .unwrap()
            .to_string();

        let task = wait_task_terminal(&app, &cookie, &task_id).await;
        assert_eq!(task["state"], "failed");
        assert_eq!(task["errors"].as_array().unwrap().len(), 1);
        assert_eq!(task["errors"][0]["path"], "/source/hello-link.txt");
        assert!(
            task["errors"][0]["message"]
                .as_str()
                .unwrap()
                .contains("不支持复制符号链接")
        );
        assert!(!target_dir.join("hello-link.txt").exists());
        assert!(!target_dir.join("hello.txt").exists());
    }

    #[tokio::test]
    async fn move_rejects_symlink_source_through_api_when_available() {
        let (root, app) = test_app("move-symlink-source-api").await;
        let files_dir = root.path().join("files");
        tokio::fs::create_dir_all(&files_dir).await.unwrap();
        let source = files_dir.join("hello.txt");
        let link = files_dir.join("hello-link.txt");
        tokio::fs::write(&source, b"move-body").await.unwrap();
        if !try_create_file_symlink(&source, &link) {
            return;
        }

        let cookie = login_cookie(&app).await;
        create_mapping(&app, &cookie, "/docs", &files_dir, true).await;

        let response = app
            .clone()
            .oneshot(json_request(
                Method::PATCH,
                "/api/file/docs/hello-link.txt",
                Some(&cookie),
                json!({ "targetPath": "/docs/moved.txt" }),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = json_body(response).await;
        assert_eq!(body["code"], "BAD_REQUEST");
        assert!(
            body["message"]
                .as_str()
                .unwrap()
                .contains("不支持移动符号链接")
        );
        assert!(source.exists());
        assert!(link.exists());
        assert!(!files_dir.join("moved.txt").exists());
    }

    #[tokio::test]
    async fn delete_rejects_symlink_target_through_api_when_available() {
        let (root, app) = test_app("delete-symlink-source-api").await;
        let files_dir = root.path().join("files");
        tokio::fs::create_dir_all(&files_dir).await.unwrap();
        let source = files_dir.join("hello.txt");
        let link = files_dir.join("hello-link.txt");
        tokio::fs::write(&source, b"delete-body").await.unwrap();
        if !try_create_file_symlink(&source, &link) {
            return;
        }

        let cookie = login_cookie(&app).await;
        create_mapping(&app, &cookie, "/docs", &files_dir, true).await;

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::DELETE,
                "/api/file/docs/hello-link.txt",
                &cookie,
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = json_body(response).await;
        assert_eq!(body["code"], "BAD_REQUEST");
        assert!(
            body["message"]
                .as_str()
                .unwrap()
                .contains("不支持删除符号链接")
        );
        assert!(source.exists());
        assert!(link.exists());
        let trash = get_json(&app, &cookie, "/api/trash").await;
        assert!(trash.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn archive_task_rejects_symlink_source_through_api_when_available() {
        let (root, app) = test_app("archive-symlink-source-api").await;
        let source_dir = root.path().join("source");
        let target_dir = root.path().join("target");
        tokio::fs::create_dir_all(&source_dir).await.unwrap();
        tokio::fs::create_dir_all(&target_dir).await.unwrap();
        let source = source_dir.join("hello.txt");
        let link = source_dir.join("hello-link.txt");
        tokio::fs::write(&source, b"archive-body").await.unwrap();
        if !try_create_file_symlink(&source, &link) {
            return;
        }

        let cookie = login_cookie(&app).await;
        create_mapping(&app, &cookie, "/source", &source_dir, true).await;
        create_mapping(&app, &cookie, "/target", &target_dir, true).await;

        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/tasks/archive",
                Some(&cookie),
                json!({
                    "sources": ["/source/hello-link.txt"],
                    "targetPath": "/target",
                    "outputName": "link.zip",
                    "format": "zip",
                    "conflictPolicy": "reject"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let task_id = json_body(response).await["id"]
            .as_str()
            .unwrap()
            .to_string();

        let task = wait_task_terminal(&app, &cookie, &task_id).await;
        assert_eq!(task["state"], "failed");
        assert_eq!(task["errors"].as_array().unwrap().len(), 1);
        assert_eq!(task["errors"][0]["path"], "/source/hello-link.txt");
        assert!(
            task["errors"][0]["message"]
                .as_str()
                .unwrap()
                .contains("不支持压缩符号链接")
        );
        assert!(!target_dir.join("link.zip").exists());
    }

    #[tokio::test]
    async fn login_allows_mapping_metadata_and_range_content() {
        let (root, app) = test_app("login-file-flow").await;
        let files_dir = root.path().join("files");
        tokio::fs::create_dir_all(&files_dir).await.unwrap();
        tokio::fs::write(files_dir.join("hello.txt"), b"hello-from-file")
            .await
            .unwrap();

        let cookie = login_cookie(&app).await;
        let mapping = json!({
            "mountPath": "/docs",
            "folderPath": path_text(&files_dir),
            "remark": "测试挂载",
            "order": 0,
            "writable": true
        });
        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/mapping",
                Some(&cookie),
                mapping,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::GET,
                "/api/file/docs",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let directory_last_modified = response
            .headers()
            .get(LAST_MODIFIED)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let body = json_body(response).await;
        assert_eq!(body["path"], "/docs");
        assert_eq!(body["file"][0]["name"], "hello.txt");

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/file/docs")
                    .header(COOKIE, &cookie)
                    .header(IF_MODIFIED_SINCE, &directory_last_modified)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_MODIFIED);

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::GET,
                "/api/file/docs/hello.txt",
                &cookie,
            ))
            .await
            .unwrap();
        let status = response.status();
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let etag = response
            .headers()
            .get(ETAG)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let last_modified = response
            .headers()
            .get(LAST_MODIFIED)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let body = json_body(response).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(content_type, "application/json");
        assert_eq!(body["name"], "hello.txt");
        assert_eq!(body["type"], "file");
        assert_eq!(body["size"], 15);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/file/docs/hello.txt")
                    .header(COOKIE, &cookie)
                    .header(IF_NONE_MATCH, "*")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_MODIFIED);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/file/docs/hello.txt")
                    .header(COOKIE, &cookie)
                    .header(IF_MODIFIED_SINCE, &last_modified)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_MODIFIED);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/file/docs/hello.txt")
                    .header(COOKIE, &cookie)
                    .header(IF_NONE_MATCH, "W/\"not-current\"")
                    .header(IF_MODIFIED_SINCE, &last_modified)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/file/docs/hello.txt")
                    .header(COOKIE, &cookie)
                    .header(IF_NONE_MATCH, etag)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_MODIFIED);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/content/docs/hello.txt")
                    .header(COOKIE, &cookie)
                    .header(RANGE, "bytes=0-4")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let body = text_body(response).await;

        assert_eq!(status, StatusCode::PARTIAL_CONTENT);
        assert_eq!(body, "hello");
    }

    #[tokio::test]
    async fn upload_and_save_over_max_bytes_return_413_through_api() {
        let (root, app) = test_app_with_config("max-upload-api", |config| {
            config.max_upload_bytes = Some(8);
        })
        .await;
        let files_dir = root.path().join("files");
        tokio::fs::create_dir_all(&files_dir).await.unwrap();
        tokio::fs::write(files_dir.join("hello.txt"), b"old")
            .await
            .unwrap();

        let cookie = login_cookie(&app).await;
        create_mapping(&app, &cookie, "/docs", &files_dir, true).await;

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::PUT)
                    .uri("/api/content/docs/hello.txt")
                    .header(COOKIE, &cookie)
                    .header("If-Match", "*")
                    .body(Body::from("too-large-save"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(json_body(response).await["code"], "PAYLOAD_TOO_LARGE");
        assert_eq!(
            tokio::fs::read_to_string(files_dir.join("hello.txt"))
                .await
                .unwrap(),
            "old"
        );

        let response = app
            .clone()
            .oneshot(multipart_upload_request(
                "/api/upload/docs",
                &cookie,
                "upload.txt",
                "too-large-upload",
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(json_body(response).await["code"], "PAYLOAD_TOO_LARGE");
        assert!(!files_dir.join("upload.txt").exists());
    }

    #[tokio::test]
    async fn directory_api_omits_totals_until_requested() {
        let (root, app) = test_app("directory-total-api").await;
        let files_dir = root.path().join("files");
        tokio::fs::create_dir_all(files_dir.join("folder-a"))
            .await
            .unwrap();
        tokio::fs::create_dir_all(files_dir.join("folder-b"))
            .await
            .unwrap();
        tokio::fs::create_dir_all(files_dir.join("folder-c"))
            .await
            .unwrap();
        tokio::fs::write(files_dir.join("a.txt"), b"a")
            .await
            .unwrap();
        tokio::fs::write(files_dir.join("b.txt"), b"b")
            .await
            .unwrap();
        tokio::fs::write(files_dir.join("c.txt"), b"c")
            .await
            .unwrap();

        let cookie = login_cookie(&app).await;
        create_mapping(&app, &cookie, "/docs", &files_dir, true).await;

        let page = get_json(&app, &cookie, "/api/file/docs?offset=2&limit=2").await;
        assert_eq!(page["offset"], 2);
        assert_eq!(page["limit"], 2);
        assert_eq!(page["hasMore"], true);
        assert!(page.get("folderTotal").is_none());
        assert!(page.get("fileTotal").is_none());
        assert_eq!(page["folder"].as_array().unwrap().len(), 1);
        assert_eq!(page["file"].as_array().unwrap().len(), 1);

        let page = get_json(
            &app,
            &cookie,
            "/api/file/docs?offset=2&limit=2&includeTotal=true",
        )
        .await;
        assert_eq!(page["folderTotal"], 3);
        assert_eq!(page["fileTotal"], 3);
        assert_eq!(page["hasMore"], true);
    }

    #[tokio::test]
    async fn directory_api_enforces_page_size_bounds() {
        let (root, app) = test_app_with_config("directory-page-limit-api", |config| {
            config.max_dir_page_size = 2;
        })
        .await;
        let files_dir = root.path().join("files");
        tokio::fs::create_dir_all(&files_dir).await.unwrap();
        for index in 0..4 {
            tokio::fs::write(files_dir.join(format!("file-{index}.txt")), b"x")
                .await
                .unwrap();
        }

        let cookie = login_cookie(&app).await;
        create_mapping(&app, &cookie, "/docs", &files_dir, true).await;

        let page = get_json(&app, &cookie, "/api/file/docs?limit=99").await;
        assert_eq!(page["limit"], 2);
        assert_eq!(page["hasMore"], true);
        assert_eq!(page["file"].as_array().unwrap().len(), 2);

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::GET,
                "/api/file/docs?limit=0",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = json_body(response).await;
        assert_eq!(body["code"], "BAD_REQUEST");
        assert_eq!(body["message"], "分页大小必须大于 0");
    }

    #[tokio::test]
    async fn directory_api_requires_full_detail_for_expensive_sort() {
        let (root, app) = test_app("directory-expensive-sort-api").await;
        let files_dir = root.path().join("files");
        tokio::fs::create_dir_all(&files_dir).await.unwrap();
        tokio::fs::write(files_dir.join("small.txt"), b"x")
            .await
            .unwrap();
        tokio::fs::write(files_dir.join("medium.txt"), b"xx")
            .await
            .unwrap();
        tokio::fs::write(files_dir.join("large.txt"), b"xxx")
            .await
            .unwrap();

        let cookie = login_cookie(&app).await;
        create_mapping(&app, &cookie, "/docs", &files_dir, true).await;

        for sort in ["size", "modified"] {
            let response = app
                .clone()
                .oneshot(empty_request_with_cookie(
                    Method::GET,
                    &format!("/api/file/docs?detail=basic&sort={sort}"),
                    &cookie,
                ))
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            let body = json_body(response).await;
            assert_eq!(body["code"], "BAD_REQUEST");
            assert!(
                body["message"]
                    .as_str()
                    .unwrap()
                    .contains("detail=basic 仅支持 sort=name")
            );
        }

        let page = get_json(
            &app,
            &cookie,
            "/api/file/docs?detail=full&sort=size&type=file&limit=2",
        )
        .await;
        let files = page["file"].as_array().unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0]["name"], "small.txt");
        assert_eq!(files[0]["size"], 1);
        assert_eq!(files[1]["name"], "medium.txt");
        assert_eq!(files[1]["size"], 2);
    }

    #[tokio::test]
    async fn head_and_invalid_range_work_through_api() {
        let (root, app) = test_app("head-range-api").await;
        let files_dir = root.path().join("files");
        tokio::fs::create_dir_all(&files_dir).await.unwrap();
        tokio::fs::write(files_dir.join("hello.txt"), b"hello-from-file")
            .await
            .unwrap();

        let cookie = login_cookie(&app).await;
        create_mapping(&app, &cookie, "/docs", &files_dir, true).await;

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::HEAD,
                "/api/content/docs/hello.txt",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get(CONTENT_LENGTH).unwrap(), "15");
        assert_eq!(response.headers().get("accept-ranges").unwrap(), "bytes");
        assert!(response.headers().get(ETAG).is_some());
        assert!(response.headers().get(CONTENT_DISPOSITION).is_none());
        assert!(
            to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap()
                .is_empty()
        );

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::HEAD)
                    .uri("/api/content/docs/hello.txt")
                    .header(COOKIE, &cookie)
                    .header(RANGE, "bytes=1-4")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(
            response.headers().get(CONTENT_RANGE).unwrap(),
            "bytes 1-4/15"
        );
        assert_eq!(response.headers().get(CONTENT_LENGTH).unwrap(), "4");
        assert!(
            to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap()
                .is_empty()
        );

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::HEAD,
                "/api/download/docs/hello.txt",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get(CONTENT_LENGTH).unwrap(), "15");
        assert!(
            response
                .headers()
                .get(CONTENT_DISPOSITION)
                .unwrap()
                .to_str()
                .unwrap()
                .contains("hello.txt")
        );
        assert!(
            to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap()
                .is_empty()
        );

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/content/docs/hello.txt")
                    .header(COOKIE, &cookie)
                    .header(RANGE, "bytes=100-120")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::RANGE_NOT_SATISFIABLE);
        let body = json_body(response).await;
        assert_eq!(body["code"], "RANGE_NOT_SATISFIABLE");
    }

    #[tokio::test]
    async fn streaming_transfer_limit_is_held_until_response_body_is_dropped() {
        let (root, app) = test_app_with_config("stream-transfer-limit-api", |config| {
            config.max_transfer_concurrency = 1;
        })
        .await;
        let files_dir = root.path().join("files");
        tokio::fs::create_dir_all(&files_dir).await.unwrap();
        tokio::fs::write(files_dir.join("hello.txt"), b"stream-body")
            .await
            .unwrap();

        let cookie = login_cookie(&app).await;
        create_mapping(&app, &cookie, "/docs", &files_dir, true).await;

        let first_response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::GET,
                "/api/content/docs/hello.txt",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(first_response.status(), StatusCode::OK);
        let metrics = get_json(&app, &cookie, "/api/metrics").await;
        assert_eq!(metrics["limits"]["activeTransfers"], 1);

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::GET,
                "/api/download/docs/hello.txt",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        let body = json_body(response).await;
        assert_eq!(body["code"], "TOO_MANY_REQUESTS");
        assert_eq!(body["message"], "文件传输并发过高，请稍后重试");

        drop(first_response);
        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::GET,
                "/api/download/docs/hello.txt",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(text_body(response).await, "stream-body");
    }

    #[tokio::test]
    async fn trusted_proxy_headers_enforce_forwarded_ip_concurrency_through_middleware() {
        let (root, app) = test_app_with_config("ip-limit-api", |config| {
            config.max_ip_concurrency = 1;
            config.max_transfer_concurrency = 2;
            config.trust_proxy_headers = true;
        })
        .await;
        let files_dir = root.path().join("files");
        tokio::fs::create_dir_all(&files_dir).await.unwrap();

        let cookie = login_cookie(&app).await;
        create_mapping(&app, &cookie, "/docs", &files_dir, true).await;

        let pending_body = Body::from_stream(stream::pending::<Result<Bytes, std::io::Error>>());
        let first_request = Request::builder()
            .method(Method::POST)
            .uri("/api/upload/docs")
            .header(COOKIE, cookie.clone())
            .header("x-forwarded-for", "10.0.0.10")
            .header(CONTENT_TYPE, "multipart/form-data; boundary=never-finishes")
            .body(pending_body)
            .unwrap();
        let first_upload = tokio::spawn({
            let app = app.clone();
            async move { app.oneshot(first_request).await }
        });

        wait_active_ip_requests(&app, &cookie, 2).await;

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie_and_ip(
                Method::GET,
                "/api/file/docs",
                &cookie,
                "10.0.0.10",
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        let body = json_body(response).await;
        assert_eq!(body["code"], "TOO_MANY_REQUESTS");
        assert_eq!(body["message"], "当前 IP 并发请求过高，请稍后重试");

        first_upload.abort();
        assert!(first_upload.await.unwrap_err().is_cancelled());
        wait_active_ip_requests(&app, &cookie, 1).await;
    }

    #[tokio::test]
    async fn protected_routes_ignore_spoofed_proxy_headers_by_default() {
        let (root, app) = test_app_with_config("ip-limit-ignore-proxy-api", |config| {
            config.max_ip_concurrency = 1;
            config.max_transfer_concurrency = 2;
        })
        .await;
        let files_dir = root.path().join("files");
        tokio::fs::create_dir_all(&files_dir).await.unwrap();

        let cookie = login_cookie(&app).await;
        create_mapping(&app, &cookie, "/docs", &files_dir, true).await;

        let pending_body = Body::from_stream(stream::pending::<Result<Bytes, std::io::Error>>());
        let first_request = with_connection_address(
            Request::builder()
                .method(Method::POST)
                .uri("/api/upload/docs")
                .header(COOKIE, cookie.clone())
                .header("x-forwarded-for", "10.0.0.10")
                .header(CONTENT_TYPE, "multipart/form-data; boundary=never-finishes")
                .body(pending_body)
                .unwrap(),
            "192.168.1.10:5000",
        );
        let first_upload = tokio::spawn({
            let app = app.clone();
            async move { app.oneshot(first_request).await }
        });

        wait_active_ip_requests(&app, &cookie, 2).await;

        let response = app
            .clone()
            .oneshot(with_connection_address(
                empty_request_with_cookie_and_ip(
                    Method::GET,
                    "/api/file/docs",
                    &cookie,
                    "10.0.0.11",
                ),
                "192.168.1.10:5001",
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        let body = json_body(response).await;
        assert_eq!(body["code"], "TOO_MANY_REQUESTS");
        assert_eq!(body["message"], "当前 IP 并发请求过高，请稍后重试");

        first_upload.abort();
        assert!(first_upload.await.unwrap_err().is_cancelled());
        wait_active_ip_requests(&app, &cookie, 1).await;
    }

    #[tokio::test]
    async fn metrics_reports_lightweight_runtime_snapshot_through_api() {
        let (root, app) = test_app_with_config("metrics-api", |config| {
            config.max_dir_concurrency = 3;
            config.max_transfer_concurrency = 5;
            config.max_ip_concurrency = 7;
        })
        .await;
        let files_dir = root.path().join("files");
        tokio::fs::create_dir_all(&files_dir).await.unwrap();
        tokio::fs::write(files_dir.join("trash-me.txt"), b"trash")
            .await
            .unwrap();

        let cookie = login_cookie(&app).await;
        create_mapping(&app, &cookie, "/docs", &files_dir, true).await;
        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::DELETE,
                "/api/file/docs/trash-me.txt",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let metrics = get_json(&app, &cookie, "/api/metrics").await;
        assert_eq!(metrics["mappings"], 1);
        assert_eq!(metrics["activeSessions"], 1);
        assert_eq!(metrics["trashEntries"], 1);
        assert_eq!(metrics["limits"]["dirScanLimit"], 3);
        assert_eq!(metrics["limits"]["activeDirScans"], 0);
        assert_eq!(metrics["limits"]["transferLimit"], 5);
        assert_eq!(metrics["limits"]["activeTransfers"], 0);
        assert_eq!(metrics["limits"]["ipLimit"], 7);
        assert_eq!(metrics["limits"]["activeIpRequests"], 1);
        assert_eq!(metrics["tasks"]["total"], 0);
        assert_eq!(metrics["tasks"]["running"], 0);
        assert_eq!(metrics["index"]["enabled"], false);
        assert_eq!(metrics["index"]["state"], "disabled");
    }

    #[tokio::test]
    async fn ready_endpoint_reports_runtime_readiness() {
        let (_root, app) = test_app("ready-ok-api").await;

        let response = app
            .oneshot(empty_request(Method::GET, "/api/ready"))
            .await
            .unwrap();
        let status = response.status();
        let body = json_body(response).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "ok");
        let checks = body["checks"].as_array().unwrap();
        for name in [
            "auth",
            "configStore",
            "mappingStore",
            "trash",
            "audit",
            "staticFiles",
        ] {
            assert!(checks.iter().any(|check| check["name"] == name));
        }
        assert!(checks.iter().all(|check| check["status"] == "ok"));
    }

    #[tokio::test]
    async fn ready_endpoint_fails_when_admin_password_missing() {
        let (_root, app) = test_app_with_config("ready-missing-password-api", |config| {
            config.initial_admin_password = None;
        })
        .await;

        let response = app
            .oneshot(empty_request(Method::GET, "/api/ready"))
            .await
            .unwrap();
        let status = response.status();
        let body = json_body(response).await;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body["status"], "notReady");
        let auth = body["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|check| check["name"] == "auth")
            .unwrap();
        assert_eq!(auth["status"], "error");
        assert!(
            auth["message"]
                .as_str()
                .unwrap()
                .contains("管理员密码尚未初始化")
        );
    }

    #[tokio::test]
    async fn download_does_not_write_audit_log() {
        let (root, app) = test_app("download-audit").await;
        let files_dir = root.path().join("files");
        tokio::fs::create_dir_all(&files_dir).await.unwrap();
        tokio::fs::write(files_dir.join("hello.txt"), b"download-body")
            .await
            .unwrap();

        let cookie = login_cookie(&app).await;
        let mapping = json!({
            "mountPath": "/docs",
            "folderPath": path_text(&files_dir),
            "remark": "测试挂载",
            "order": 0,
            "writable": true
        });
        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/mapping",
                Some(&cookie),
                mapping,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let audit_path = root.path().join("data/audit.jsonl");
        let before = tokio::fs::read_to_string(&audit_path).await.unwrap();

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::GET,
                "/api/download/docs/hello.txt",
                &cookie,
            ))
            .await
            .unwrap();
        let status = response.status();
        let body = text_body(response).await;
        let after = tokio::fs::read_to_string(&audit_path).await.unwrap();

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "download-body");
        assert_eq!(before, after);
        assert!(!after.contains("\"action\":\"download\""));
    }

    #[tokio::test]
    async fn audit_records_writes_and_skips_high_frequency_reads() {
        let (root, app) = test_app("audit-boundary").await;
        let files_dir = root.path().join("files");
        tokio::fs::create_dir_all(&files_dir).await.unwrap();
        tokio::fs::write(files_dir.join("hello.txt"), b"read-body")
            .await
            .unwrap();

        let cookie = login_cookie(&app).await;
        create_mapping(&app, &cookie, "/docs", &files_dir, true).await;
        let records = read_audit_records(&root).await;
        assert_audit_action(&records, "login", None);

        let audit_before_reads = audit_text(&root).await;
        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::GET,
                "/api/file/docs",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let _ = json_body(response).await;

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::GET,
                "/api/content/docs/hello.txt",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(text_body(response).await, "read-body");

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::GET,
                "/api/download/docs/hello.txt",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(text_body(response).await, "read-body");
        assert_eq!(audit_before_reads, audit_text(&root).await);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::PUT)
                    .uri("/api/content/docs/hello.txt")
                    .header(COOKIE, &cookie)
                    .header("If-Match", "*")
                    .body(Body::from("saved-body"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(json_body(response).await["path"], "/docs/hello.txt");

        let response = app
            .clone()
            .oneshot(multipart_upload_request(
                "/api/upload/docs",
                &cookie,
                "upload.txt",
                "upload-body",
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let body = json_body(response).await;
        assert_eq!(body["files"][0]["path"], "/docs/upload.txt");
        assert_eq!(
            tokio::fs::read_to_string(files_dir.join("upload.txt"))
                .await
                .unwrap(),
            "upload-body"
        );

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::DELETE,
                "/api/file/docs/upload.txt",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let trash = get_json(&app, &cookie, "/api/trash").await;
        let record_id = trash[0]["id"].as_str().unwrap();
        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::POST,
                &format!("/api/trash/{record_id}/restore"),
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            json_body(response).await["restoredVirtualPath"],
            "/docs/upload.txt"
        );

        let records = read_audit_records(&root).await;
        assert_eq!(audit_action_count(&records, "save"), 1);
        assert_eq!(audit_action_count(&records, "upload"), 1);
        assert_eq!(audit_action_count(&records, "delete"), 1);
        assert_eq!(audit_action_count(&records, "trash.restore"), 1);
        assert_audit_action(&records, "save", Some("/docs/hello.txt"));
        assert_audit_action(&records, "upload", None);
        assert_audit_action(&records, "delete", Some("/docs/upload.txt"));
        assert_audit_action(&records, "trash.restore", Some("/docs/upload.txt"));
        assert_no_audit_action(&records, "download");
        assert_no_audit_action(&records, "preview");
        assert_no_audit_action(&records, "list");
    }

    #[tokio::test]
    async fn audit_cleanup_removes_old_rotated_logs_through_api() {
        let (root, app) = test_app_with_config("audit-cleanup-api", |config| {
            config.audit_retention_files = 1;
        })
        .await;
        let cookie = login_cookie(&app).await;
        let data_dir = root.path().join("data");
        tokio::fs::write(data_dir.join("audit.10.jsonl"), "old 10\n")
            .await
            .unwrap();
        tokio::fs::write(data_dir.join("audit.20.jsonl"), "old 20\n")
            .await
            .unwrap();

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::POST,
                "/api/audit/cleanup",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = json_body(response).await;

        assert_eq!(body["removed"], 1);
        assert!(!data_dir.join("audit.10.jsonl").exists());
        assert!(data_dir.join("audit.20.jsonl").exists());
        let records = read_audit_records(&root).await;
        assert_audit_action(&records, "audit.cleanup", None);
    }

    #[tokio::test]
    async fn copy_task_reports_partial_success_and_errors() {
        let (root, app) = test_app("copy-task-partial-failure").await;
        let files_dir = root.path().join("files");
        let target_dir = files_dir.join("target");
        tokio::fs::create_dir_all(&target_dir).await.unwrap();
        tokio::fs::write(files_dir.join("ok.txt"), b"copy-body")
            .await
            .unwrap();

        let cookie = login_cookie(&app).await;
        let mapping = json!({
            "mountPath": "/docs",
            "folderPath": path_text(&files_dir),
            "remark": "测试挂载",
            "order": 0,
            "writable": true
        });
        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/mapping",
                Some(&cookie),
                mapping,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/tasks/copy",
                Some(&cookie),
                json!({
                    "sources": ["/docs/ok.txt", "/docs/missing.txt"],
                    "targetPath": "/docs/target",
                    "conflictPolicy": "reject"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let task_id = json_body(response).await["id"]
            .as_str()
            .unwrap()
            .to_string();

        let task = wait_task_terminal(&app, &cookie, &task_id).await;

        assert_eq!(task["state"], "failed");
        assert_eq!(task["processedItems"], 1);
        assert_eq!(task["totalItems"], 2);
        assert_eq!(task["errors"].as_array().unwrap().len(), 1);
        assert_eq!(task["errors"][0]["path"], "/docs/missing.txt");
        assert!(target_dir.join("ok.txt").is_file());
    }

    #[tokio::test]
    async fn task_create_rejects_blank_paths_without_creating_task() {
        let (_root, app) = test_app("task-blank-path-api").await;
        let cookie = login_cookie(&app).await;
        let cases = [
            (
                "/api/tasks/copy",
                json!({
                    "sources": [" "],
                    "targetPath": "/docs/target"
                }),
                "复制任务 sources 包含空路径",
            ),
            (
                "/api/tasks/move",
                json!({
                    "sources": ["/docs/a.txt"],
                    "targetPath": " "
                }),
                "移动任务 targetPath 不能为空",
            ),
            (
                "/api/tasks/delete",
                json!({
                    "paths": [" "]
                }),
                "删除任务 paths 包含空路径",
            ),
            (
                "/api/tasks/archive",
                json!({
                    "sources": ["/docs/source"],
                    "targetPath": "",
                    "format": "zip"
                }),
                "压缩任务 targetPath 不能为空",
            ),
            (
                "/api/tasks/archive",
                json!({
                    "sources": ["/docs/source"],
                    "targetPath": "/docs",
                    "format": "zip",
                    "outputName": " "
                }),
                "压缩任务 outputName 不能为空",
            ),
            (
                "/api/tasks/extract",
                json!({
                    "sourcePath": "",
                    "targetPath": "/docs"
                }),
                "解压任务 sourcePath 不能为空",
            ),
            (
                "/api/tasks/extract",
                json!({
                    "sourcePath": "/docs/source.zip",
                    "targetPath": "/docs",
                    "folderName": "bad/name"
                }),
                "解压任务 folderName 无效: 路径不能包含 .. 或路径分隔符",
            ),
        ];

        for (uri, body, expected_message) in cases {
            let response = app
                .clone()
                .oneshot(json_request(Method::POST, uri, Some(&cookie), body))
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            let body = json_body(response).await;
            assert_eq!(body["code"], "BAD_REQUEST");
            assert_eq!(body["message"], expected_message);
        }

        let tasks = get_json(&app, &cookie, "/api/tasks").await;
        assert!(tasks.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn task_cleanup_removes_finished_tasks_through_api() {
        let (root, app) = test_app("task-cleanup-api").await;
        let files_dir = root.path().join("files");
        let target_dir = files_dir.join("target");
        tokio::fs::create_dir_all(&target_dir).await.unwrap();
        tokio::fs::write(files_dir.join("ok.txt"), b"copy-body")
            .await
            .unwrap();

        let cookie = login_cookie(&app).await;
        create_mapping(&app, &cookie, "/docs", &files_dir, true).await;
        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/tasks/copy",
                Some(&cookie),
                json!({
                    "sources": ["/docs/ok.txt"],
                    "targetPath": "/docs/target",
                    "conflictPolicy": "reject"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let task_id = json_body(response).await["id"]
            .as_str()
            .unwrap()
            .to_string();

        let task = wait_task_terminal(&app, &cookie, &task_id).await;
        assert_eq!(task["state"], "completed");
        let tasks_before_cleanup = get_json(&app, &cookie, "/api/tasks").await;
        assert_eq!(tasks_before_cleanup.as_array().unwrap().len(), 1);

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::POST,
                "/api/tasks/cleanup",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = json_body(response).await;
        let tasks_after_cleanup = get_json(&app, &cookie, "/api/tasks").await;
        let records = read_audit_records(&root).await;

        assert_eq!(body["removed"], 1);
        assert!(tasks_after_cleanup.as_array().unwrap().is_empty());
        assert_audit_action(&records, "task.cleanup", None);
    }

    #[tokio::test]
    async fn task_cancel_rejects_finished_task_through_api() {
        let (root, app) = test_app("task-cancel-finished-api").await;
        let files_dir = root.path().join("files");
        let target_dir = files_dir.join("target");
        tokio::fs::create_dir_all(&target_dir).await.unwrap();
        tokio::fs::write(files_dir.join("ok.txt"), b"copy-body")
            .await
            .unwrap();

        let cookie = login_cookie(&app).await;
        create_mapping(&app, &cookie, "/docs", &files_dir, true).await;

        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/tasks/copy",
                Some(&cookie),
                json!({
                    "sources": ["/docs/ok.txt"],
                    "targetPath": "/docs/target",
                    "conflictPolicy": "reject"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let task_id = json_body(response).await["id"]
            .as_str()
            .unwrap()
            .to_string();

        let completed = wait_task_terminal(&app, &cookie, &task_id).await;
        assert_eq!(completed["state"], "completed");
        assert_eq!(completed["cancelled"], false);

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::POST,
                &format!("/api/tasks/{task_id}/cancel"),
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = json_body(response).await;
        assert_eq!(body["code"], "CONFLICT");
        assert_eq!(body["message"], "任务已结束，不能取消");

        let task = get_task_json(&app, &cookie, &task_id).await;
        assert_eq!(task["state"], "completed");
        assert_eq!(task["cancelled"], false);
    }

    #[tokio::test]
    async fn copy_task_cancel_endpoint_stops_running_task() {
        let (root, app) = test_app_with_config("copy-task-cancel", |config| {
            config.task_speed_limit_bytes_per_sec = Some(1);
        })
        .await;
        let files_dir = root.path().join("files");
        let target_dir = files_dir.join("target");
        tokio::fs::create_dir_all(&target_dir).await.unwrap();
        tokio::fs::write(files_dir.join("large.bin"), vec![b'a'; 300 * 1024])
            .await
            .unwrap();
        tokio::fs::write(target_dir.join("large.bin"), b"old-target")
            .await
            .unwrap();

        let cookie = login_cookie(&app).await;
        let mapping = json!({
            "mountPath": "/docs",
            "folderPath": path_text(&files_dir),
            "remark": "测试挂载",
            "order": 0,
            "writable": true
        });
        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/mapping",
                Some(&cookie),
                mapping,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/tasks/copy",
                Some(&cookie),
                json!({
                    "sources": ["/docs/large.bin"],
                    "targetPath": "/docs/target",
                    "conflictPolicy": "overwrite"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let task_id = json_body(response).await["id"]
            .as_str()
            .unwrap()
            .to_string();

        let running = wait_task_processed_bytes_at_least(&app, &cookie, &task_id, 1).await;
        assert_eq!(running["state"], "running");

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::POST,
                &format!("/api/tasks/{task_id}/cancel"),
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let task = wait_task_terminal(&app, &cookie, &task_id).await;
        assert_eq!(task["state"], "cancelled");
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        assert_eq!(
            tokio::fs::read_to_string(target_dir.join("large.bin"))
                .await
                .unwrap(),
            "old-target"
        );
        let mut entries = tokio::fs::read_dir(&target_dir).await.unwrap();
        let mut names = Vec::new();
        while let Some(entry) = entries.next_entry().await.unwrap() {
            names.push(entry.file_name().to_string_lossy().to_string());
        }
        assert_eq!(names, vec!["large.bin"]);
    }

    #[tokio::test]
    async fn queued_move_task_cancel_does_not_move_source() {
        let (root, app) = test_app_with_config("move-task-cancel", |config| {
            config.max_task_concurrency = 1;
            config.task_speed_limit_bytes_per_sec = Some(1);
        })
        .await;
        let files_dir = root.path().join("files");
        let target_dir = files_dir.join("target");
        tokio::fs::create_dir_all(&target_dir).await.unwrap();
        tokio::fs::write(files_dir.join("large.bin"), vec![b'a'; 300 * 1024])
            .await
            .unwrap();
        tokio::fs::write(files_dir.join("move-me.txt"), b"move-body")
            .await
            .unwrap();

        let cookie = login_cookie(&app).await;
        create_mapping(&app, &cookie, "/docs", &files_dir, true).await;

        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/tasks/copy",
                Some(&cookie),
                json!({
                    "sources": ["/docs/large.bin"],
                    "targetPath": "/docs/target",
                    "conflictPolicy": "reject"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let copy_task_id = json_body(response).await["id"]
            .as_str()
            .unwrap()
            .to_string();
        wait_task_processed_bytes_at_least(&app, &cookie, &copy_task_id, 1).await;

        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/tasks/move",
                Some(&cookie),
                json!({
                    "sources": ["/docs/move-me.txt"],
                    "targetPath": "/docs/target",
                    "conflictPolicy": "reject"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let move_task_id = json_body(response).await["id"]
            .as_str()
            .unwrap()
            .to_string();

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::POST,
                &format!("/api/tasks/{move_task_id}/cancel"),
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(json_body(response).await["state"], "cancelled");

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::POST,
                &format!("/api/tasks/{copy_task_id}/cancel"),
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let _copy_task = wait_task_terminal(&app, &cookie, &copy_task_id).await;
        let move_task = wait_task_terminal(&app, &cookie, &move_task_id).await;

        assert_eq!(move_task["state"], "cancelled");
        assert_eq!(move_task["processedItems"], 0);
        assert!(files_dir.join("move-me.txt").is_file());
        assert!(!target_dir.join("move-me.txt").exists());
    }

    #[tokio::test]
    async fn archive_and_extract_tasks_work_through_api() {
        let (root, app) = test_app("archive-extract-api").await;
        let files_dir = root.path().join("files");
        let source_dir = files_dir.join("source");
        tokio::fs::create_dir_all(&source_dir).await.unwrap();
        tokio::fs::write(source_dir.join("hello.txt"), b"archive-body")
            .await
            .unwrap();

        let cookie = login_cookie(&app).await;
        let mapping = json!({
            "mountPath": "/docs",
            "folderPath": path_text(&files_dir),
            "remark": "测试挂载",
            "order": 0,
            "writable": true
        });
        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/mapping",
                Some(&cookie),
                mapping,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/tasks/archive",
                Some(&cookie),
                json!({
                    "sources": ["/docs/source"],
                    "targetPath": "/docs",
                    "format": "zip",
                    "outputName": "source.zip",
                    "conflictPolicy": "reject"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let archive_task_id = json_body(response).await["id"]
            .as_str()
            .unwrap()
            .to_string();

        let archive_task = wait_task_terminal(&app, &cookie, &archive_task_id).await;
        assert_eq!(archive_task["state"], "completed");
        assert!(files_dir.join("source.zip").is_file());

        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/tasks/extract",
                Some(&cookie),
                json!({
                    "sourcePath": "/docs/source.zip",
                    "targetPath": "/docs",
                    "folderName": "unzipped",
                    "conflictPolicy": "reject"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let extract_task_id = json_body(response).await["id"]
            .as_str()
            .unwrap()
            .to_string();

        let extract_task = wait_task_terminal(&app, &cookie, &extract_task_id).await;
        assert_eq!(extract_task["state"], "completed");
        let extracted = tokio::fs::read_to_string(files_dir.join("unzipped/source/hello.txt"))
            .await
            .unwrap();
        assert_eq!(extracted, "archive-body");
    }

    #[tokio::test]
    async fn tar_gz_archive_and_extract_tasks_work_through_api() {
        let (root, app) = test_app("tar-gz-archive-extract-api").await;
        let files_dir = root.path().join("files");
        let source_dir = files_dir.join("source");
        tokio::fs::create_dir_all(&source_dir).await.unwrap();
        tokio::fs::write(source_dir.join("hello.txt"), b"tar-body")
            .await
            .unwrap();

        let cookie = login_cookie(&app).await;
        let mapping = json!({
            "mountPath": "/docs",
            "folderPath": path_text(&files_dir),
            "remark": "测试挂载",
            "order": 0,
            "writable": true
        });
        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/mapping",
                Some(&cookie),
                mapping,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/tasks/archive",
                Some(&cookie),
                json!({
                    "sources": ["/docs/source"],
                    "targetPath": "/docs",
                    "format": "tarGz",
                    "outputName": "source.tar.gz",
                    "conflictPolicy": "reject"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let archive_task_id = json_body(response).await["id"]
            .as_str()
            .unwrap()
            .to_string();

        let archive_task = wait_task_terminal(&app, &cookie, &archive_task_id).await;
        assert_eq!(archive_task["state"], "completed");
        assert!(files_dir.join("source.tar.gz").is_file());

        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/tasks/extract",
                Some(&cookie),
                json!({
                    "sourcePath": "/docs/source.tar.gz",
                    "targetPath": "/docs",
                    "folderName": "untarred",
                    "conflictPolicy": "reject"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let extract_task_id = json_body(response).await["id"]
            .as_str()
            .unwrap()
            .to_string();

        let extract_task = wait_task_terminal(&app, &cookie, &extract_task_id).await;
        assert_eq!(extract_task["state"], "completed");
        let extracted = tokio::fs::read_to_string(files_dir.join("untarred/source/hello.txt"))
            .await
            .unwrap();
        assert_eq!(extracted, "tar-body");
    }

    #[tokio::test]
    async fn zip_archive_and_extract_large_file_track_streamed_bytes() {
        let (root, app) = test_app("zip-large-archive-extract-api").await;
        let files_dir = root.path().join("files");
        let source_dir = files_dir.join("source");
        tokio::fs::create_dir_all(&source_dir).await.unwrap();
        let content = large_test_bytes(600 * 1024 + 17);
        tokio::fs::write(source_dir.join("large.bin"), &content)
            .await
            .unwrap();

        let cookie = login_cookie(&app).await;
        create_mapping(&app, &cookie, "/docs", &files_dir, true).await;

        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/tasks/archive",
                Some(&cookie),
                json!({
                    "sources": ["/docs/source"],
                    "targetPath": "/docs",
                    "format": "zip",
                    "outputName": "large-source.zip",
                    "conflictPolicy": "reject"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let archive_task_id = json_body(response).await["id"]
            .as_str()
            .unwrap()
            .to_string();

        let archive_task = wait_task_terminal(&app, &cookie, &archive_task_id).await;
        assert_eq!(archive_task["state"], "completed");
        assert_eq!(
            archive_task["processedBytes"].as_u64().unwrap(),
            content.len() as u64
        );
        assert!(files_dir.join("large-source.zip").is_file());

        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/tasks/extract",
                Some(&cookie),
                json!({
                    "sourcePath": "/docs/large-source.zip",
                    "targetPath": "/docs",
                    "folderName": "unzipped-large",
                    "conflictPolicy": "reject"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let extract_task_id = json_body(response).await["id"]
            .as_str()
            .unwrap()
            .to_string();

        let extract_task = wait_task_terminal(&app, &cookie, &extract_task_id).await;
        assert_eq!(extract_task["state"], "completed");
        assert_eq!(
            extract_task["processedBytes"].as_u64().unwrap(),
            content.len() as u64
        );
        let extracted = tokio::fs::read(files_dir.join("unzipped-large/source/large.bin"))
            .await
            .unwrap();
        assert_eq!(extracted, content);
    }

    #[tokio::test]
    async fn tar_gz_archive_and_extract_large_file_track_streamed_bytes() {
        let (root, app) = test_app("tar-gz-large-archive-extract-api").await;
        let files_dir = root.path().join("files");
        let source_dir = files_dir.join("source");
        tokio::fs::create_dir_all(&source_dir).await.unwrap();
        let content = large_test_bytes(600 * 1024 + 19);
        tokio::fs::write(source_dir.join("large.bin"), &content)
            .await
            .unwrap();

        let cookie = login_cookie(&app).await;
        create_mapping(&app, &cookie, "/docs", &files_dir, true).await;

        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/tasks/archive",
                Some(&cookie),
                json!({
                    "sources": ["/docs/source"],
                    "targetPath": "/docs",
                    "format": "tarGz",
                    "outputName": "large-source.tar.gz",
                    "conflictPolicy": "reject"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let archive_task_id = json_body(response).await["id"]
            .as_str()
            .unwrap()
            .to_string();

        let archive_task = wait_task_terminal(&app, &cookie, &archive_task_id).await;
        assert_eq!(archive_task["state"], "completed");
        assert_eq!(
            archive_task["processedBytes"].as_u64().unwrap(),
            content.len() as u64
        );
        assert!(files_dir.join("large-source.tar.gz").is_file());

        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/tasks/extract",
                Some(&cookie),
                json!({
                    "sourcePath": "/docs/large-source.tar.gz",
                    "targetPath": "/docs",
                    "folderName": "untarred-large",
                    "conflictPolicy": "reject"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let extract_task_id = json_body(response).await["id"]
            .as_str()
            .unwrap()
            .to_string();

        let extract_task = wait_task_terminal(&app, &cookie, &extract_task_id).await;
        assert_eq!(extract_task["state"], "completed");
        assert_eq!(
            extract_task["processedBytes"].as_u64().unwrap(),
            content.len() as u64
        );
        let extracted = tokio::fs::read(files_dir.join("untarred-large/source/large.bin"))
            .await
            .unwrap();
        assert_eq!(extracted, content);
    }

    #[tokio::test]
    async fn extract_task_rejects_entry_over_configured_depth() {
        let (root, app) = test_app_with_config("extract-depth-limit-api", |config| {
            config.max_extract_depth = 2;
        })
        .await;
        let files_dir = root.path().join("files");
        tokio::fs::create_dir_all(&files_dir).await.unwrap();
        let archive_path = files_dir.join("too-deep.zip");
        create_test_zip(&archive_path, &[("a/b/c.txt", b"deep".as_slice())]);

        let cookie = login_cookie(&app).await;
        create_mapping(&app, &cookie, "/docs", &files_dir, true).await;

        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/tasks/extract",
                Some(&cookie),
                json!({
                    "sourcePath": "/docs/too-deep.zip",
                    "targetPath": "/docs",
                    "folderName": "out",
                    "conflictPolicy": "reject"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let task_id = json_body(response).await["id"]
            .as_str()
            .unwrap()
            .to_string();

        let task = wait_task_terminal(&app, &cookie, &task_id).await;
        assert_eq!(task["state"], "failed");
        assert!(
            task["errors"][0]["message"]
                .as_str()
                .unwrap()
                .contains("压缩包条目路径过深")
        );
        assert!(!files_dir.join("out").exists());
    }

    #[tokio::test]
    async fn search_api_rebuild_recent_and_incremental_updates() {
        let (root, app) = test_app_with_config("search-api-flow", |config| {
            config.index_enabled = true;
            config.index_scan_delay_ms = 0;
        })
        .await;
        let files_dir = root.path().join("files");
        tokio::fs::create_dir_all(&files_dir).await.unwrap();
        tokio::fs::write(files_dir.join("alpha-needle.txt"), b"search-body")
            .await
            .unwrap();

        let cookie = login_cookie(&app).await;
        let mapping = json!({
            "mountPath": "/docs",
            "folderPath": path_text(&files_dir),
            "remark": "测试挂载",
            "order": 0,
            "writable": true
        });
        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/mapping",
                Some(&cookie),
                mapping,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let search_before_rebuild = get_json(&app, &cookie, "/api/search?q=alpha-needle").await;
        assert_eq!(search_before_rebuild["total"], 0);

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::POST,
                "/api/index/rebuild",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        wait_index_entries_at_least(&app, &cookie, 1).await;

        let rebuilt_search = get_json(&app, &cookie, "/api/search?q=alpha-needle").await;
        assert_eq!(rebuilt_search["total"], 1);
        assert_eq!(rebuilt_search["items"][0]["path"], "/docs/alpha-needle.txt");

        let recent = get_json(&app, &cookie, "/api/recent?limit=1").await;
        assert_eq!(recent.as_array().unwrap().len(), 1);
        assert_eq!(recent[0]["path"], "/docs/alpha-needle.txt");

        let response = app
            .clone()
            .oneshot(json_request(
                Method::PATCH,
                "/api/file/docs/alpha-needle.txt",
                Some(&cookie),
                json!({ "targetPath": "/docs/beta-needle.txt" }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let old_name_search = get_json(&app, &cookie, "/api/search?q=alpha-needle").await;
        let renamed_search = get_json(&app, &cookie, "/api/search?q=beta-needle").await;
        assert_eq!(old_name_search["total"], 0);
        assert_eq!(renamed_search["total"], 1);

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::DELETE,
                "/api/file/docs/beta-needle.txt",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let deleted_search = get_json(&app, &cookie, "/api/search?q=beta-needle").await;
        assert_eq!(deleted_search["total"], 0);

        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/file/docs",
                Some(&cookie),
                json!({ "type": "file", "name": "fresh-needle.txt" }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let created_search = get_json(&app, &cookie, "/api/search?q=fresh-needle").await;
        assert_eq!(created_search["total"], 1);
        assert_eq!(created_search["items"][0]["path"], "/docs/fresh-needle.txt");
    }

    #[tokio::test]
    async fn search_api_enforces_limit_bounds() {
        let (root, app) = test_app_with_config("search-limit-api", |config| {
            config.index_enabled = true;
            config.index_scan_delay_ms = 0;
            config.max_dir_page_size = 2;
        })
        .await;
        let files_dir = root.path().join("files");
        tokio::fs::create_dir_all(&files_dir).await.unwrap();
        for index in 0..4 {
            tokio::fs::write(files_dir.join(format!("needle-{index}.txt")), b"search")
                .await
                .unwrap();
        }

        let cookie = login_cookie(&app).await;
        create_mapping(&app, &cookie, "/docs", &files_dir, true).await;

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::POST,
                "/api/index/rebuild",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        wait_index_entries_at_least(&app, &cookie, 4).await;

        let search = get_json(&app, &cookie, "/api/search?q=needle&limit=99").await;
        assert_eq!(search["limit"], 2);
        assert_eq!(search["total"], 4);
        assert_eq!(search["items"].as_array().unwrap().len(), 2);

        let recent = get_json(&app, &cookie, "/api/recent?limit=99").await;
        assert_eq!(recent.as_array().unwrap().len(), 2);

        for uri in ["/api/search?q=needle&limit=0", "/api/recent?limit=0"] {
            let response = app
                .clone()
                .oneshot(empty_request_with_cookie(Method::GET, uri, &cookie))
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            let body = json_body(response).await;
            assert_eq!(body["code"], "BAD_REQUEST");
            assert_eq!(body["message"], "分页大小必须大于 0");
        }
    }

    #[tokio::test]
    async fn search_index_endpoints_return_stable_errors_for_disabled_and_idle_states() {
        let (_root, app) = test_app("search-index-disabled-api").await;
        let cookie = login_cookie(&app).await;

        let status = get_json(&app, &cookie, "/api/index/status").await;
        assert_eq!(status["enabled"], false);
        assert_eq!(status["state"], "disabled");

        for uri in ["/api/index/rebuild", "/api/index/cancel"] {
            let response = app
                .clone()
                .oneshot(empty_request_with_cookie(Method::POST, uri, &cookie))
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            let body = json_body(response).await;
            assert_eq!(body["code"], "BAD_REQUEST");
            assert_eq!(body["message"], "搜索索引未启用");
        }

        let (_root, app) = test_app_with_config("search-index-idle-cancel-api", |config| {
            config.index_enabled = true;
        })
        .await;
        let cookie = login_cookie(&app).await;
        let status = get_json(&app, &cookie, "/api/index/status").await;
        assert_eq!(status["enabled"], true);
        assert_eq!(status["state"], "idle");

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::POST,
                "/api/index/cancel",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = json_body(response).await;
        assert_eq!(body["code"], "CONFLICT");
        assert_eq!(body["message"], "当前没有正在重建的索引");
    }

    #[tokio::test]
    async fn search_index_cancel_endpoint_stops_running_rebuild() {
        let (root, app) = test_app_with_config("search-cancel-api", |config| {
            config.index_enabled = true;
            config.index_scan_delay_ms = 50;
        })
        .await;
        let files_dir = root.path().join("files");
        create_nested_search_tree(&files_dir, 20).await;

        let cookie = login_cookie(&app).await;
        let mapping = json!({
            "mountPath": "/docs",
            "folderPath": path_text(&files_dir),
            "remark": "测试挂载",
            "order": 0,
            "writable": true
        });
        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/mapping",
                Some(&cookie),
                mapping,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::POST,
                "/api/index/rebuild",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        wait_index_state(&app, &cookie, "scanning").await;

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::POST,
                "/api/index/cancel",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);

        let status = wait_index_state(&app, &cookie, "cancelled").await;
        assert_eq!(status["lastError"], "索引重建已取消");
    }

    #[tokio::test]
    async fn search_rebuild_rejects_duplicate_running_request() {
        let (root, app) = test_app_with_config("search-duplicate-rebuild-api", |config| {
            config.index_enabled = true;
            config.index_scan_delay_ms = 50;
        })
        .await;
        let files_dir = root.path().join("files");
        create_nested_search_tree(&files_dir, 20).await;

        let cookie = login_cookie(&app).await;
        create_mapping(&app, &cookie, "/docs", &files_dir, true).await;

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::POST,
                "/api/index/rebuild",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        wait_index_state(&app, &cookie, "scanning").await;

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::POST,
                "/api/index/rebuild",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = json_body(response).await;
        assert_eq!(body["code"], "CONFLICT");
        assert_eq!(body["message"], "索引正在重建");

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::POST,
                "/api/index/cancel",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        wait_index_state(&app, &cookie, "cancelled").await;
    }

    #[tokio::test]
    async fn trash_delete_list_and_restore_work_through_api() {
        let (root, app) = test_app("trash-api-flow").await;
        let files_dir = root.path().join("files");
        tokio::fs::create_dir_all(&files_dir).await.unwrap();
        tokio::fs::write(files_dir.join("hello.txt"), b"deleted-body")
            .await
            .unwrap();

        let cookie = login_cookie(&app).await;
        let mapping = json!({
            "mountPath": "/docs",
            "folderPath": path_text(&files_dir),
            "remark": "测试挂载",
            "order": 0,
            "writable": true
        });
        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/mapping",
                Some(&cookie),
                mapping,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::DELETE,
                "/api/file/docs/hello.txt",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(!files_dir.join("hello.txt").exists());

        let trash = get_json(&app, &cookie, "/api/trash").await;
        let records = trash.as_array().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0]["originalVirtualPath"], "/docs/hello.txt");
        assert_eq!(records[0]["sizeBytes"], 12);
        let record_id = records[0]["id"].as_str().unwrap().to_string();

        tokio::fs::write(files_dir.join("hello.txt"), b"current-body")
            .await
            .unwrap();
        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::POST,
                &format!("/api/trash/{record_id}/restore"),
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let restored = json_body(response).await;

        assert_eq!(restored["restoredVirtualPath"], "/docs/hello (1).txt");
        assert_eq!(
            tokio::fs::read_to_string(files_dir.join("hello.txt"))
                .await
                .unwrap(),
            "current-body"
        );
        assert_eq!(
            tokio::fs::read_to_string(files_dir.join("hello (1).txt"))
                .await
                .unwrap(),
            "deleted-body"
        );
        let trash_after_restore = get_json(&app, &cookie, "/api/trash").await;
        assert!(trash_after_restore.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn trash_restore_reject_conflict_policy_preserves_record_through_api() {
        let (root, app) = test_app("trash-restore-reject-conflict-api").await;
        let files_dir = root.path().join("files");
        tokio::fs::create_dir_all(&files_dir).await.unwrap();
        tokio::fs::write(files_dir.join("hello.txt"), b"deleted-body")
            .await
            .unwrap();

        let cookie = login_cookie(&app).await;
        create_mapping(&app, &cookie, "/docs", &files_dir, true).await;

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::DELETE,
                "/api/file/docs/hello.txt",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let trash = get_json(&app, &cookie, "/api/trash").await;
        let records = trash.as_array().unwrap();
        assert_eq!(records.len(), 1);
        let record_id = records[0]["id"].as_str().unwrap().to_string();

        tokio::fs::write(files_dir.join("hello.txt"), b"current-body")
            .await
            .unwrap();
        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::POST,
                &format!("/api/trash/{record_id}/restore?conflictPolicy=reject"),
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let error = json_body(response).await;
        assert_eq!(error["code"], "CONFLICT");

        assert_eq!(
            tokio::fs::read_to_string(files_dir.join("hello.txt"))
                .await
                .unwrap(),
            "current-body"
        );
        let trash_after_restore = get_json(&app, &cookie, "/api/trash").await;
        let records = trash_after_restore.as_array().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0]["id"], record_id);
    }

    #[tokio::test]
    async fn trash_restore_overwrite_conflict_policy_replaces_target_through_api() {
        let (root, app) = test_app("trash-restore-overwrite-conflict-api").await;
        let files_dir = root.path().join("files");
        tokio::fs::create_dir_all(&files_dir).await.unwrap();
        tokio::fs::write(files_dir.join("hello.txt"), b"deleted-body")
            .await
            .unwrap();

        let cookie = login_cookie(&app).await;
        create_mapping(&app, &cookie, "/docs", &files_dir, true).await;

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::DELETE,
                "/api/file/docs/hello.txt",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let trash = get_json(&app, &cookie, "/api/trash").await;
        let records = trash.as_array().unwrap();
        assert_eq!(records.len(), 1);
        let record_id = records[0]["id"].as_str().unwrap().to_string();

        tokio::fs::write(files_dir.join("hello.txt"), b"current-body")
            .await
            .unwrap();
        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::POST,
                &format!("/api/trash/{record_id}/restore?conflict=overwrite"),
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let restored = json_body(response).await;
        assert_eq!(restored["restoredVirtualPath"], "/docs/hello.txt");

        assert_eq!(
            tokio::fs::read_to_string(files_dir.join("hello.txt"))
                .await
                .unwrap(),
            "deleted-body"
        );
        assert!(!files_dir.join("hello (1).txt").exists());
        let trash_after_restore = get_json(&app, &cookie, "/api/trash").await;
        assert!(trash_after_restore.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn trash_purge_and_empty_work_through_api() {
        let (root, app) = test_app("trash-purge-empty-api").await;
        let files_dir = root.path().join("files");
        tokio::fs::create_dir_all(&files_dir).await.unwrap();
        tokio::fs::write(files_dir.join("purge.txt"), b"purge-body")
            .await
            .unwrap();
        tokio::fs::write(files_dir.join("empty.txt"), b"empty-body")
            .await
            .unwrap();

        let cookie = login_cookie(&app).await;
        let mapping = json!({
            "mountPath": "/docs",
            "folderPath": path_text(&files_dir),
            "remark": "测试挂载",
            "order": 0,
            "writable": true
        });
        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/mapping",
                Some(&cookie),
                mapping,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::DELETE,
                "/api/file/docs/purge.txt",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let trash = get_json(&app, &cookie, "/api/trash").await;
        let record = trash.as_array().unwrap().first().unwrap();
        let record_id = record["id"].as_str().unwrap();
        let trash_payload = PathBuf::from(record["trashPath"].as_str().unwrap());
        assert!(trash_payload.exists());

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::DELETE,
                &format!("/api/trash/{record_id}"),
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert!(!trash_payload.exists());
        let trash_after_purge = get_json(&app, &cookie, "/api/trash").await;
        assert!(trash_after_purge.as_array().unwrap().is_empty());

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::DELETE,
                "/api/file/docs/empty.txt",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let trash = get_json(&app, &cookie, "/api/trash").await;
        let record = trash.as_array().unwrap().first().unwrap();
        let trash_payload = PathBuf::from(record["trashPath"].as_str().unwrap());
        assert!(trash_payload.exists());

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::POST,
                "/api/trash/empty",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = json_body(response).await;
        assert_eq!(body["removed"], 1);
        assert!(!trash_payload.exists());
        let trash_after_empty = get_json(&app, &cookie, "/api/trash").await;
        assert!(trash_after_empty.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn trash_cleanup_applies_retention_policy_through_api() {
        let (root, app) = test_app_with_config("trash-cleanup-api", |config| {
            config.trash_max_bytes = Some(0);
        })
        .await;
        let files_dir = root.path().join("files");
        tokio::fs::create_dir_all(&files_dir).await.unwrap();
        tokio::fs::write(files_dir.join("cleanup.txt"), b"cleanup-body")
            .await
            .unwrap();

        let cookie = login_cookie(&app).await;
        let mapping = json!({
            "mountPath": "/docs",
            "folderPath": path_text(&files_dir),
            "remark": "测试挂载",
            "order": 0,
            "writable": true
        });
        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/mapping",
                Some(&cookie),
                mapping,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::DELETE,
                "/api/file/docs/cleanup.txt",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::POST,
                "/api/trash/cleanup",
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = json_body(response).await;
        assert_eq!(body["removed"], 1);
        let trash_after_cleanup = get_json(&app, &cookie, "/api/trash").await;
        assert!(trash_after_cleanup.as_array().unwrap().is_empty());
    }

    async fn test_app(name: &str) -> (TempRoot, Router) {
        test_app_with_config(name, |_| {}).await
    }

    async fn test_app_with_config(
        name: &str,
        configure: impl FnOnce(&mut AppConfig),
    ) -> (TempRoot, Router) {
        let root = TempRoot::new(name);
        tokio::fs::create_dir_all(root.path().join("static"))
            .await
            .unwrap();
        let mut config = AppConfig {
            bind_address: "127.0.0.1".to_string(),
            port: 0,
            mapping_file: root.path().join("data/mappings.json"),
            config_file: root.path().join("data/config.json"),
            initial_admin_password: Some("test-password".to_string()),
            trash_dir: root.path().join("data/trash"),
            static_dir: root.path().join("static"),
            cors_allowed_origins: Vec::new(),
            trust_proxy_headers: false,
            max_edit_bytes: 2 * 1024 * 1024,
            editable_extensions: Vec::new(),
            editable_mime_types: Vec::new(),
            max_upload_bytes: None,
            max_dir_page_size: 2_000,
            audit_file: root.path().join("data/audit.jsonl"),
            audit_max_bytes: Some(10 * 1024 * 1024),
            audit_retention_files: 8,
            max_dir_concurrency: 4,
            max_transfer_concurrency: 8,
            max_ip_concurrency: 16,
            max_task_concurrency: 2,
            task_history_limit: 200,
            task_speed_limit_bytes_per_sec: None,
            max_extract_bytes: None,
            max_extract_files: None,
            max_extract_depth: 64,
            index_enabled: false,
            index_rebuild_on_startup: false,
            index_scan_delay_ms: 2,
            trash_retention_days: None,
            trash_max_bytes: None,
            conflict_policy: ConflictPolicy::AutoRename,
        };
        configure(&mut config);
        let app = build(config).await.unwrap();
        (root, app)
    }

    async fn login_cookie(app: &Router) -> String {
        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/auth/login",
                None,
                json!({ "password": "test-password" }),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        response
            .headers()
            .get(SET_COOKIE)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }

    async fn create_mapping(
        app: &Router,
        cookie: &str,
        mount_path: &str,
        path: &Path,
        writable: bool,
    ) {
        let response = app
            .clone()
            .oneshot(json_request(
                Method::POST,
                "/api/mapping",
                Some(cookie),
                json!({
                    "mountPath": mount_path,
                    "folderPath": path_text(path),
                    "remark": "测试挂载",
                    "order": 0,
                    "writable": writable
                }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    fn empty_request(method: Method, uri: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .unwrap()
    }

    fn empty_request_with_cookie(method: Method, uri: &str, cookie: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .header(COOKIE, cookie)
            .body(Body::empty())
            .unwrap()
    }

    fn empty_request_with_cookie_and_ip(
        method: Method,
        uri: &str,
        cookie: &str,
        ip: &str,
    ) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .header(COOKIE, cookie)
            .header("x-forwarded-for", ip)
            .body(Body::empty())
            .unwrap()
    }

    fn with_connection_address(mut request: Request<Body>, address: &str) -> Request<Body> {
        request.extensions_mut().insert(ConnectInfo(
            address.parse::<std::net::SocketAddr>().unwrap(),
        ));
        request
    }

    fn json_request(
        method: Method,
        uri: &str,
        cookie: Option<&str>,
        value: Value,
    ) -> Request<Body> {
        let mut builder = Request::builder()
            .method(method)
            .uri(uri)
            .header(CONTENT_TYPE, "application/json");
        if let Some(cookie) = cookie {
            builder = builder.header(COOKIE, cookie);
        }
        builder
            .body(Body::from(serde_json::to_vec(&value).unwrap()))
            .unwrap()
    }

    fn multipart_upload_request(
        uri: &str,
        cookie: &str,
        file_name: &str,
        content: &str,
    ) -> Request<Body> {
        let boundary = "web-file-browser-test-boundary";
        let body = format!(
            "--{boundary}\r\n\
             Content-Disposition: form-data; name=\"file\"; filename=\"{file_name}\"\r\n\
             Content-Type: text/plain\r\n\r\n\
             {content}\r\n\
             --{boundary}--\r\n"
        );
        Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header(COOKIE, cookie)
            .header(
                CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .unwrap()
    }

    async fn json_body(response: Response<Body>) -> Value {
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    async fn text_body(response: Response<Body>) -> String {
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    async fn get_json(app: &Router, cookie: &str, uri: &str) -> Value {
        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(Method::GET, uri, cookie))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        json_body(response).await
    }

    async fn wait_active_ip_requests(app: &Router, cookie: &str, expected: u64) -> Value {
        for _ in 0..100 {
            let response = app
                .clone()
                .oneshot(with_connection_address(
                    empty_request_with_cookie_and_ip(
                        Method::GET,
                        "/api/metrics",
                        cookie,
                        "10.0.0.20",
                    ),
                    "192.168.1.20:5000",
                ))
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let body = json_body(response).await;
            if body["limits"]["activeIpRequests"].as_u64() == Some(expected) {
                return body;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        panic!("单 IP 并发计数未在预期时间内变为 {expected}");
    }

    async fn audit_text(root: &TempRoot) -> String {
        tokio::fs::read_to_string(root.path().join("data/audit.jsonl"))
            .await
            .unwrap_or_default()
    }

    async fn read_audit_records(root: &TempRoot) -> Vec<Value> {
        audit_text(root)
            .await
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    fn audit_action_count(records: &[Value], action: &str) -> usize {
        records
            .iter()
            .filter(|record| record.get("action").and_then(Value::as_str) == Some(action))
            .count()
    }

    fn assert_audit_action(records: &[Value], action: &str, path: Option<&str>) {
        assert!(
            records.iter().any(|record| {
                let action_matches = record.get("action").and_then(Value::as_str) == Some(action);
                let path_matches = path.is_none_or(|expected| {
                    record.get("path").and_then(Value::as_str) == Some(expected)
                });
                action_matches && path_matches
            }),
            "缺少审计记录: action={action}, path={path:?}"
        );
    }

    fn assert_no_audit_action(records: &[Value], action: &str) {
        assert!(
            records
                .iter()
                .all(|record| record.get("action").and_then(Value::as_str) != Some(action)),
            "不应写入审计动作: {action}"
        );
    }

    async fn get_task_json(app: &Router, cookie: &str, task_id: &str) -> Value {
        get_json(app, cookie, &format!("/api/tasks/{task_id}")).await
    }

    async fn wait_task_terminal(app: &Router, cookie: &str, task_id: &str) -> Value {
        for _ in 0..50 {
            let body = get_task_json(app, cookie, task_id).await;
            if matches!(
                body["state"].as_str(),
                Some("completed" | "failed" | "cancelled")
            ) {
                return body;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        panic!("任务未在预期时间内结束: {task_id}");
    }

    async fn wait_task_processed_bytes_at_least(
        app: &Router,
        cookie: &str,
        task_id: &str,
        min_bytes: u64,
    ) -> Value {
        for _ in 0..100 {
            let body = get_task_json(app, cookie, task_id).await;
            if body["processedBytes"].as_u64().unwrap_or(0) >= min_bytes {
                return body;
            }
            if matches!(
                body["state"].as_str(),
                Some("completed" | "failed" | "cancelled")
            ) {
                panic!("任务过早结束: {task_id}");
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        panic!("任务未在预期时间内开始复制: {task_id}");
    }

    async fn wait_index_entries_at_least(app: &Router, cookie: &str, min_entries: u64) -> Value {
        for _ in 0..100 {
            let body = get_json(app, cookie, "/api/index/status").await;
            if body["state"] == "idle"
                && body["indexedEntries"].as_u64().unwrap_or(0) >= min_entries
            {
                return body;
            }
            if matches!(body["state"].as_str(), Some("failed" | "cancelled")) {
                panic!("索引重建未成功完成");
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        panic!("索引未在预期时间内完成");
    }

    async fn wait_index_state(app: &Router, cookie: &str, expected_state: &str) -> Value {
        for _ in 0..100 {
            let body = get_json(app, cookie, "/api/index/status").await;
            if body["state"] == expected_state {
                return body;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        panic!("索引未在预期时间内进入状态: {expected_state}");
    }

    async fn create_nested_search_tree(root: &Path, count: usize) {
        tokio::fs::create_dir_all(root).await.unwrap();
        for index in 0..count {
            let dir = root.join(format!("dir-{index:02}"));
            tokio::fs::create_dir_all(&dir).await.unwrap();
            tokio::fs::write(dir.join("file.txt"), b"search")
                .await
                .unwrap();
        }
    }

    fn path_text(path: &Path) -> String {
        path.to_string_lossy().to_string()
    }

    fn large_test_bytes(size: usize) -> Vec<u8> {
        (0..size)
            .map(|index| ((index * 31 + 7) % 251) as u8)
            .collect()
    }

    fn create_test_zip(path: &Path, files: &[(&str, &[u8])]) {
        let file = fs::File::create(path).unwrap();
        let mut writer = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        for (name, content) in files {
            writer.start_file(*name, options).unwrap();
            writer.write_all(content).unwrap();
        }
        writer.finish().unwrap();
    }

    fn try_create_file_symlink(source: &Path, link: &Path) -> bool {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(source, link).is_ok()
        }
        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_file(source, link).is_ok()
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = (source, link);
            false
        }
    }

    struct TempRoot {
        path: PathBuf,
    }

    impl TempRoot {
        fn new(name: &str) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!("web-file-browser-{name}-{nonce}"));
            std::fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempRoot {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}
