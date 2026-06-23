use super::common::*;

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
