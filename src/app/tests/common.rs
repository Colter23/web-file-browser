pub(super) use axum::{
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
pub(super) use bytes::Bytes;
pub(super) use futures_util::stream;
pub(super) use serde_json::{Value, json};
pub(super) use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
pub(super) use tower::ServiceExt;
pub(super) use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

pub(super) use crate::{
    app::{build, build_cors_layer},
    config::AppConfig,
    error::AppError,
    models::ConflictPolicy,
};

pub(super) async fn test_app(name: &str) -> (TempRoot, Router) {
    test_app_with_config(name, |_| {}).await
}

pub(super) async fn test_app_with_config(
    name: &str,
    configure: impl FnOnce(&mut AppConfig),
) -> (TempRoot, Router) {
    let root = TempRoot::new(name);
    tokio::fs::create_dir_all(root.path().join("static"))
        .await
        .unwrap();
    tokio::fs::write(
        root.path().join("static/index.html"),
        b"<div id=\"app\"></div>",
    )
    .await
    .unwrap();
    let mut config = AppConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0,
        mapping_file: root.path().join("data/mappings.json"),
        config_file: root.path().join("data/config.json"),
        auth_file: root.path().join("data/auth.json"),
        favorites_file: root.path().join("data/favorites.json"),
        trash_dir: root.path().join("data/trash"),
        static_dir: root.path().join("static"),
        cors_allowed_origins: Vec::new(),
        trust_proxy_headers: false,
        auth_session_ttl_seconds: 7 * 24 * 60 * 60,
        auth_secure_cookie: false,
        max_edit_bytes: 2 * 1024 * 1024,
        editable_extensions: Vec::new(),
        editable_mime_types: Vec::new(),
        max_upload_bytes: None,
        max_dir_page_size: 2_000,
        audit_file: root.path().join("data/audit.jsonl"),
        audit_enabled: true,
        audit_max_bytes: Some(10 * 1024 * 1024),
        audit_retention_files: 8,
        max_dir_concurrency: 4,
        max_transfer_concurrency: 8,
        max_ip_concurrency: 16,
        max_task_concurrency: 2,
        task_history_limit: 200,
        task_speed_limit_bytes_per_sec: None,
        max_archive_bytes: None,
        max_archive_files: None,
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

pub(super) async fn login_cookie(app: &Router) -> String {
    let setup_response = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/api/auth/setup",
            None,
            json!({ "password": "test-password" }),
        ))
        .await
        .unwrap();

    match setup_response.status() {
        StatusCode::OK => {
            let setup_cookie = setup_response
                .headers()
                .get(SET_COOKIE)
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            let logout_response = app
                .clone()
                .oneshot(empty_request_with_cookie(
                    Method::POST,
                    "/api/auth/logout",
                    &setup_cookie,
                ))
                .await
                .unwrap();
            assert_eq!(logout_response.status(), StatusCode::OK);
        }
        StatusCode::CONFLICT => {}
        status => panic!("首次设置密码接口返回了非预期状态: {status}"),
    }

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

pub(super) async fn create_mapping(
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

pub(super) fn empty_request(method: Method, uri: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

pub(super) fn empty_request_with_cookie(method: Method, uri: &str, cookie: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(COOKIE, cookie)
        .body(Body::empty())
        .unwrap()
}

pub(super) fn empty_request_with_cookie_and_ip(
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

pub(super) fn with_connection_address(mut request: Request<Body>, address: &str) -> Request<Body> {
    request.extensions_mut().insert(ConnectInfo(
        address.parse::<std::net::SocketAddr>().unwrap(),
    ));
    request
}

pub(super) fn json_request(
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

pub(super) fn multipart_upload_request(
    uri: &str,
    cookie: &str,
    file_name: &str,
    content: &str,
) -> Request<Body> {
    multipart_upload_request_bytes(uri, cookie, file_name, content.as_bytes())
}

pub(super) fn multipart_upload_request_bytes(
    uri: &str,
    cookie: &str,
    file_name: &str,
    content: &[u8],
) -> Request<Body> {
    multipart_upload_request_many(uri, cookie, &[(file_name, content)])
}

pub(super) fn multipart_upload_request_many(
    uri: &str,
    cookie: &str,
    files: &[(&str, &[u8])],
) -> Request<Body> {
    let boundary = "web-file-browser-test-boundary";
    let footer = format!("--{boundary}--\r\n");
    let mut body = Vec::new();
    for (file_name, content) in files {
        let header = format!(
            "--{boundary}\r\n\
                 Content-Disposition: form-data; name=\"file\"; filename=\"{file_name}\"\r\n\
                 Content-Type: application/octet-stream\r\n\r\n"
        );
        body.extend_from_slice(header.as_bytes());
        body.extend_from_slice(content);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(footer.as_bytes());
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

pub(super) async fn json_body(response: Response<Body>) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

pub(super) async fn text_body(response: Response<Body>) -> String {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

pub(super) async fn get_json(app: &Router, cookie: &str, uri: &str) -> Value {
    let response = app
        .clone()
        .oneshot(empty_request_with_cookie(Method::GET, uri, cookie))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    json_body(response).await
}

pub(super) async fn wait_active_ip_requests(app: &Router, cookie: &str, expected: u64) -> Value {
    for _ in 0..100 {
        let response = app
            .clone()
            .oneshot(with_connection_address(
                empty_request_with_cookie_and_ip(Method::GET, "/api/metrics", cookie, "10.0.0.20"),
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

pub(super) async fn audit_text(root: &TempRoot) -> String {
    tokio::fs::read_to_string(root.path().join("data/audit.jsonl"))
        .await
        .unwrap_or_default()
}

pub(super) async fn read_audit_records(root: &TempRoot) -> Vec<Value> {
    audit_text(root)
        .await
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

pub(super) fn audit_action_count(records: &[Value], action: &str) -> usize {
    records
        .iter()
        .filter(|record| record.get("action").and_then(Value::as_str) == Some(action))
        .count()
}

pub(super) fn assert_audit_action(records: &[Value], action: &str, path: Option<&str>) {
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

pub(super) fn assert_no_audit_action(records: &[Value], action: &str) {
    assert!(
        records
            .iter()
            .all(|record| record.get("action").and_then(Value::as_str) != Some(action)),
        "不应写入审计动作: {action}"
    );
}

pub(super) async fn get_task_json(app: &Router, cookie: &str, task_id: &str) -> Value {
    get_json(app, cookie, &format!("/api/tasks/{task_id}")).await
}

pub(super) async fn wait_task_terminal(app: &Router, cookie: &str, task_id: &str) -> Value {
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

pub(super) async fn wait_task_processed_bytes_at_least(
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

pub(super) async fn wait_index_entries_at_least(
    app: &Router,
    cookie: &str,
    min_entries: u64,
) -> Value {
    for _ in 0..100 {
        let body = get_json(app, cookie, "/api/index/status").await;
        if body["state"] == "idle" && body["indexedEntries"].as_u64().unwrap_or(0) >= min_entries {
            return body;
        }
        if matches!(body["state"].as_str(), Some("failed" | "cancelled")) {
            panic!("索引重建未成功完成");
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    panic!("索引未在预期时间内完成");
}

pub(super) async fn wait_index_state(app: &Router, cookie: &str, expected_state: &str) -> Value {
    for _ in 0..100 {
        let body = get_json(app, cookie, "/api/index/status").await;
        if body["state"] == expected_state {
            return body;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    panic!("索引未在预期时间内进入状态: {expected_state}");
}

pub(super) async fn create_nested_search_tree(root: &Path, count: usize) {
    tokio::fs::create_dir_all(root).await.unwrap();
    for index in 0..count {
        let dir = root.join(format!("dir-{index:02}"));
        tokio::fs::create_dir_all(&dir).await.unwrap();
        tokio::fs::write(dir.join("file.txt"), b"search")
            .await
            .unwrap();
    }
}

pub(super) fn path_text(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

pub(super) fn large_test_bytes(size: usize) -> Vec<u8> {
    (0..size)
        .map(|index| ((index * 31 + 7) % 251) as u8)
        .collect()
}

pub(super) fn create_test_zip(path: &Path, files: &[(&str, &[u8])]) {
    let file = fs::File::create(path).unwrap();
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    for (name, content) in files {
        writer.start_file(*name, options).unwrap();
        writer.write_all(content).unwrap();
    }
    writer.finish().unwrap();
}

pub(super) fn try_create_file_symlink(source: &Path, link: &Path) -> bool {
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

pub(super) struct TempRoot {
    path: PathBuf,
}

impl TempRoot {
    pub(super) fn new(name: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("web-file-browser-{name}-{nonce}"));
        std::fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
