use super::common::*;

#[tokio::test]
async fn settings_patch_updates_upload_limit_and_persists_config() {
    let (root, app) = test_app("settings-patch-api").await;
    let cookie = login_cookie(&app).await;
    let files_dir = root.path().join("files");
    fs::create_dir_all(&files_dir).unwrap();
    create_mapping(&app, &cookie, "/docs", &files_dir, true).await;

    let response = app
        .clone()
        .oneshot(json_request(
            Method::PATCH,
            "/api/settings",
            Some(&cookie),
            json!({
                "runtime": {
                    "maxUploadBytes": 4,
                    "maxDirPageSize": 20,
                    "editableExtensions": [".TXT", " md "],
                    "conflictPolicy": "reject"
                }
            }),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(body["runtime"]["maxUploadBytes"], 4);
    assert_eq!(body["runtime"]["maxDirPageSize"], 20);
    assert_eq!(body["runtime"]["editableExtensions"], json!(["txt", "md"]));
    assert_eq!(body["runtime"]["conflictPolicy"], "reject");
    assert!(
        body["startup"]["configFile"]
            .as_str()
            .unwrap()
            .ends_with("config.json")
    );
    assert!(body["envLocked"].as_array().is_some());
    assert!(body["restartRequiredFields"].as_array().is_some());

    let rejected_upload = app
        .clone()
        .oneshot(multipart_upload_request(
            "/api/upload/docs",
            &cookie,
            "too-large.txt",
            "12345",
        ))
        .await
        .unwrap();
    assert_eq!(rejected_upload.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert!(!files_dir.join("too-large.txt").exists());

    let text = fs::read_to_string(root.path().join("data/config.json")).unwrap();
    let persisted: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(persisted["limits"]["maxUploadBytes"], 4);
    assert_eq!(persisted["limits"]["maxDirPageSize"], 20);
    assert_eq!(
        persisted["editor"]["editableExtensions"],
        json!(["txt", "md"])
    );
    assert_eq!(persisted["conflictPolicy"], "reject");
}

#[tokio::test]
async fn settings_reload_reads_config_file_and_applies_runtime() {
    let (root, app) = test_app("settings-reload-api").await;
    let cookie = login_cookie(&app).await;
    fs::write(
        root.path().join("data/config.json"),
        r#"{"limits":{"maxDirPageSize":3},"conflictPolicy":"overwrite"}"#,
    )
    .unwrap();

    let response = app
        .clone()
        .oneshot(empty_request_with_cookie(
            Method::POST,
            "/api/settings/reload",
            &cookie,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(body["runtime"]["maxDirPageSize"], 3);
    assert_eq!(body["runtime"]["conflictPolicy"], "overwrite");
}

#[tokio::test]
async fn settings_patch_rejects_startup_fields() {
    let (_root, app) = test_app("settings-startup-reject-api").await;
    let cookie = login_cookie(&app).await;

    let response = app
        .clone()
        .oneshot(json_request(
            Method::PATCH,
            "/api/settings",
            Some(&cookie),
            json!({
                "startup": {
                    "port": 18080
                }
            }),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
