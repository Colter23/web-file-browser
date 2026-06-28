use super::common::*;

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
    assert_eq!(task["errors"][0]["code"], "FORBIDDEN");
    assert_eq!(task["errors"][0]["reason"], "MOUNT_READONLY");
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
            "sources",
        ),
        (
            "/api/tasks/move",
            json!({
                "sources": ["/docs/a.txt"],
                "targetPath": " "
            }),
            "移动任务 targetPath 不能为空",
            "targetPath",
        ),
        (
            "/api/tasks/delete",
            json!({
                "paths": [" "]
            }),
            "删除任务 paths 包含空路径",
            "paths",
        ),
        (
            "/api/tasks/archive",
            json!({
                "sources": ["/docs/source"],
                "targetPath": "",
                "format": "zip"
            }),
            "压缩任务 targetPath 不能为空",
            "targetPath",
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
            "outputName",
        ),
        (
            "/api/tasks/extract",
            json!({
                "sourcePath": "",
                "targetPath": "/docs"
            }),
            "解压任务 sourcePath 不能为空",
            "sourcePath",
        ),
        (
            "/api/tasks/extract",
            json!({
                "sourcePath": "/docs/source.zip",
                "targetPath": "/docs",
                "folderName": "bad/name"
            }),
            "解压任务 folderName 无效: 路径不能包含 .. 或路径分隔符",
            "folderName",
        ),
    ];

    for (uri, body, expected_message, expected_field) in cases {
        let response = app
            .clone()
            .oneshot(json_request(Method::POST, uri, Some(&cookie), body))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = json_body(response).await;
        assert_eq!(body["code"], "BAD_REQUEST");
        assert_eq!(body["message"], expected_message);
        assert_eq!(body["params"]["field"], expected_field);
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
