use super::common::*;

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
            empty_request_with_cookie_and_ip(Method::GET, "/api/file/docs", &cookie, "10.0.0.11"),
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
        "authStore",
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
async fn ready_endpoint_stays_ready_when_waiting_for_first_password_setup() {
    let (_root, app) = test_app("ready-waiting-auth-setup-api").await;

    let response = app
        .oneshot(empty_request(Method::GET, "/api/ready"))
        .await
        .unwrap();
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
    let auth = body["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|check| check["name"] == "auth")
        .unwrap();
    assert_eq!(auth["status"], "ok");
    assert!(
        auth["message"]
            .as_str()
            .unwrap()
            .contains("管理员密码尚未初始化")
    );
}
