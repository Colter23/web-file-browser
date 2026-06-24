use super::common::*;

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
async fn internal_mount_trash_directory_is_not_accessible_through_api() {
    let (root, app) = test_app("internal-trash-hidden-api").await;
    let files_dir = root.path().join("files");
    tokio::fs::create_dir_all(&files_dir).await.unwrap();
    tokio::fs::write(files_dir.join("hello.txt"), b"delete-body")
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
    assert!(files_dir.join(".web-file-browser-trash").is_dir());

    let response = app
        .clone()
        .oneshot(empty_request_with_cookie(
            Method::GET,
            "/api/file/docs/.web-file-browser-trash",
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let metadata = get_json(&app, &cookie, "/api/file/docs?includeHidden=true").await;
    let folders = metadata["folder"].as_array().unwrap();
    assert!(
        folders
            .iter()
            .all(|folder| folder["name"] != ".web-file-browser-trash")
    );
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
async fn multipart_upload_over_axum_default_limit_streams_through_api() {
    let (root, app) = test_app_with_config("large-multipart-upload-api", |config| {
        config.max_upload_bytes = Some(4 * 1024 * 1024);
    })
    .await;
    let files_dir = root.path().join("files");
    tokio::fs::create_dir_all(&files_dir).await.unwrap();

    let cookie = login_cookie(&app).await;
    create_mapping(&app, &cookie, "/docs", &files_dir, true).await;

    let body = vec![b'a'; 3 * 1024 * 1024];
    let response = app
        .clone()
        .oneshot(multipart_upload_request_bytes(
            "/api/upload/docs",
            &cookie,
            "large.bin",
            &body,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(
        json_body(response).await["files"][0]["path"],
        "/docs/large.bin"
    );
    let metadata = tokio::fs::metadata(files_dir.join("large.bin"))
        .await
        .unwrap();
    assert_eq!(metadata.len(), body.len() as u64);
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
async fn edit_mode_content_returns_full_body_after_text_probe() {
    let (root, app) = test_app("edit-mode-content-api").await;
    let files_dir = root.path().join("files");
    let text_dir = files_dir.join("文本Demo");
    tokio::fs::create_dir_all(&text_dir).await.unwrap();
    let content = "第一行中文\n第二行编辑内容";
    tokio::fs::write(text_dir.join("文本1.txt"), content)
        .await
        .unwrap();

    let cookie = login_cookie(&app).await;
    create_mapping(&app, &cookie, "/Demo", &files_dir, true).await;

    let response = app
        .clone()
        .oneshot(empty_request_with_cookie(
            Method::GET,
            "/api/content/Demo/%E6%96%87%E6%9C%ACDemo/%E6%96%87%E6%9C%AC1.txt?mode=edit",
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let content_length = response
        .headers()
        .get(CONTENT_LENGTH)
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(bytes.len(), content_length);
    assert_eq!(String::from_utf8(bytes.to_vec()).unwrap(), content);
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
