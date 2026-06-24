use super::common::*;

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
    let trash_path = PathBuf::from(records[0]["trashPath"].as_str().unwrap());
    assert!(
        trash_path.canonicalize().unwrap().starts_with(
            files_dir
                .join(".web-file-browser-trash")
                .canonicalize()
                .unwrap()
        )
    );
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
async fn permanent_delete_removes_file_without_trash_record() {
    let (root, app) = test_app("permanent-delete-api").await;
    let files_dir = root.path().join("files");
    tokio::fs::create_dir_all(&files_dir).await.unwrap();
    tokio::fs::write(files_dir.join("gone.txt"), b"delete-now")
        .await
        .unwrap();

    let cookie = login_cookie(&app).await;
    create_mapping(&app, &cookie, "/docs", &files_dir, true).await;

    let response = app
        .clone()
        .oneshot(empty_request_with_cookie(
            Method::DELETE,
            "/api/file/docs/gone.txt?permanent=true",
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(!files_dir.join("gone.txt").exists());
    assert!(!files_dir.join(".web-file-browser-trash").exists());

    let trash = get_json(&app, &cookie, "/api/trash").await;
    assert!(trash.as_array().unwrap().is_empty());
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
async fn trash_batch_restore_and_purge_report_partial_errors() {
    let (root, app) = test_app("trash-batch-api").await;
    let files_dir = root.path().join("files");
    tokio::fs::create_dir_all(&files_dir).await.unwrap();
    tokio::fs::write(files_dir.join("restore-a.txt"), b"restore-a")
        .await
        .unwrap();
    tokio::fs::write(files_dir.join("restore-b.txt"), b"restore-b")
        .await
        .unwrap();
    tokio::fs::write(files_dir.join("purge-a.txt"), b"purge-a")
        .await
        .unwrap();
    tokio::fs::write(files_dir.join("purge-b.txt"), b"purge-b")
        .await
        .unwrap();

    let cookie = login_cookie(&app).await;
    create_mapping(&app, &cookie, "/docs", &files_dir, true).await;

    for name in [
        "restore-a.txt",
        "restore-b.txt",
        "purge-a.txt",
        "purge-b.txt",
    ] {
        let response = app
            .clone()
            .oneshot(empty_request_with_cookie(
                Method::DELETE,
                &format!("/api/file/docs/{name}"),
                &cookie,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    let trash = get_json(&app, &cookie, "/api/trash").await;
    let records = trash.as_array().unwrap();
    let id_for = |name: &str| {
        records
            .iter()
            .find(|record| record["originalVirtualPath"] == format!("/docs/{name}"))
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let restore_a = id_for("restore-a.txt");
    let restore_b = id_for("restore-b.txt");
    let purge_a = id_for("purge-a.txt");
    let purge_b = id_for("purge-b.txt");

    let response = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/api/trash/batch/restore",
            Some(&cookie),
            json!({
                "ids": [restore_a, restore_b, "missing-restore"],
                "conflictPolicy": "reject"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["success"], 2);
    assert_eq!(body["failed"], 1);
    assert_eq!(body["errors"][0]["id"], "missing-restore");
    assert!(files_dir.join("restore-a.txt").is_file());
    assert!(files_dir.join("restore-b.txt").is_file());

    let response = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/api/trash/batch/purge",
            Some(&cookie),
            json!({
                "ids": [purge_a, purge_b, "missing-purge"]
            }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["success"], 2);
    assert_eq!(body["failed"], 1);
    assert_eq!(body["errors"][0]["id"], "missing-purge");
    assert!(!files_dir.join("purge-a.txt").exists());
    assert!(!files_dir.join("purge-b.txt").exists());

    let trash_after_batch = get_json(&app, &cookie, "/api/trash").await;
    assert!(trash_after_batch.as_array().unwrap().is_empty());
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
