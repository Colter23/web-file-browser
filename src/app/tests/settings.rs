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
                    "authSessionTtlSeconds": 3600,
                    "authSecureCookie": true,
                    "maxUploadBytes": 4,
                    "maxDirPageSize": 20,
                    "editableExtensions": [".TXT", " md "],
                    "maxArchiveBytes": 1024,
                    "maxArchiveFiles": 10,
                    "auditEnabled": false,
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
    assert_eq!(body["runtime"]["authSessionTtlSeconds"], 3600);
    assert_eq!(body["runtime"]["authSecureCookie"], true);
    assert_eq!(body["runtime"]["maxDirPageSize"], 20);
    assert_eq!(body["runtime"]["editableExtensions"], json!(["txt", "md"]));
    assert_eq!(body["runtime"]["maxArchiveBytes"], 1024);
    assert_eq!(body["runtime"]["maxArchiveFiles"], 10);
    assert_eq!(body["runtime"]["auditEnabled"], false);
    assert_eq!(body["runtime"]["conflictPolicy"], "reject");
    assert_eq!(body["restartPending"], false);
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
    assert_eq!(persisted["auth"]["sessionTtlSeconds"], 3600);
    assert_eq!(persisted["auth"]["secureCookie"], true);
    assert_eq!(persisted["limits"]["maxUploadBytes"], 4);
    assert_eq!(persisted["limits"]["maxDirPageSize"], 20);
    assert_eq!(
        persisted["editor"]["editableExtensions"],
        json!(["txt", "md"])
    );
    assert_eq!(persisted["archive"]["maxArchiveBytes"], 1024);
    assert_eq!(persisted["archive"]["maxArchiveFiles"], 10);
    assert_eq!(persisted["audit"]["enabled"], false);
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
async fn settings_patch_saves_startup_fields_for_next_restart() {
    let (root, app) = test_app("settings-startup-patch-api").await;
    let cookie = login_cookie(&app).await;

    let response = app
        .clone()
        .oneshot(json_request(
            Method::PATCH,
            "/api/settings",
            Some(&cookie),
            json!({
                "startup": {
                    "bindAddress": "127.0.0.1",
                    "port": 18080,
                    "staticDir": "ui-custom/dist",
                    "corsAllowedOrigins": ["http://localhost:5173"],
                    "trustProxyHeaders": true,
                    "indexRebuildOnStartup": true
                }
            }),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(body["startup"]["bindAddress"], "127.0.0.1");
    assert_eq!(body["startup"]["port"], 18080);
    assert_eq!(body["startup"]["staticDir"], "ui-custom/dist");
    assert_eq!(body["startup"]["trustProxyHeaders"], true);
    assert_eq!(body["startup"]["indexRebuildOnStartup"], true);
    assert_eq!(body["activeStartup"]["port"], 0);
    assert_eq!(body["activeStartup"]["trustProxyHeaders"], false);
    assert_eq!(body["restartPending"], true);
    assert!(
        body["restartPendingFields"]
            .as_array()
            .unwrap()
            .contains(&json!("startup.port"))
    );

    let text = fs::read_to_string(root.path().join("data/config.json")).unwrap();
    let persisted: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(persisted["server"]["bind"], "127.0.0.1");
    assert_eq!(persisted["server"]["port"], 18080);
    assert_eq!(persisted["server"]["staticDir"], "ui-custom/dist");
    assert_eq!(
        persisted["server"]["corsAllowedOrigins"],
        json!(["http://localhost:5173"])
    );
    assert_eq!(persisted["server"]["trustProxyHeaders"], true);
    assert_eq!(persisted["index"]["rebuildOnStartup"], true);
}

#[tokio::test]
async fn settings_patch_rejects_combined_request_without_partial_runtime_apply() {
    let (root, app) = test_app("settings-combined-patch-atomic-api").await;
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
                    "maxDirPageSize": 20
                },
                "startup": {
                    "configFile": "data/other-config.json"
                }
            }),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let accepted_upload = app
        .clone()
        .oneshot(multipart_upload_request(
            "/api/upload/docs",
            &cookie,
            "still-allowed.txt",
            "12345",
        ))
        .await
        .unwrap();
    assert_eq!(accepted_upload.status(), StatusCode::CREATED);
    assert_eq!(
        fs::read_to_string(files_dir.join("still-allowed.txt")).unwrap(),
        "12345"
    );

    let config_path = root.path().join("data/config.json");
    if config_path.exists() {
        let text = fs::read_to_string(config_path).unwrap();
        let persisted: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(persisted["limits"]["maxUploadBytes"], Value::Null);
        assert_eq!(persisted["limits"]["maxDirPageSize"], Value::Null);
    }
}

#[tokio::test]
async fn settings_patch_rejects_config_file_field() {
    let (_root, app) = test_app("settings-config-file-reject-api").await;
    let cookie = login_cookie(&app).await;

    let response = app
        .clone()
        .oneshot(json_request(
            Method::PATCH,
            "/api/settings",
            Some(&cookie),
            json!({
                "startup": {
                    "configFile": "data/other-config.json"
                }
            }),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
