use super::common::*;

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
