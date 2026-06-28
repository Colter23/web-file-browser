use super::common::*;

#[tokio::test]
async fn mapping_reorder_updates_list_and_root_order() {
    let (root, app) = test_app("mapping-reorder").await;
    let first_dir = root.path().join("first");
    let second_dir = root.path().join("second");
    tokio::fs::create_dir_all(&first_dir).await.unwrap();
    tokio::fs::create_dir_all(&second_dir).await.unwrap();

    let cookie = login_cookie(&app).await;
    let first_id = create_mapping_with_order(&app, &cookie, "/first", &first_dir, 10).await;
    let second_id = create_mapping_with_order(&app, &cookie, "/second", &second_dir, 20).await;

    let before = get_json(&app, &cookie, "/api/mapping").await;
    assert_eq!(before[0]["mountPath"], "/first");
    assert_eq!(before[1]["mountPath"], "/second");

    let response = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/api/mapping/reorder",
            Some(&cookie),
            json!({
                "items": [
                    { "id": first_id, "order": 30 },
                    { "id": second_id, "order": 5 }
                ]
            }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let after = get_json(&app, &cookie, "/api/mapping").await;
    assert_eq!(after[0]["id"], second_id);
    assert_eq!(after[0]["mountPath"], "/second");
    assert_eq!(after[0]["order"], 5);
    assert_eq!(after[1]["id"], first_id);
    assert_eq!(after[1]["mountPath"], "/first");
    assert_eq!(after[1]["order"], 30);

    let root_node = get_json(&app, &cookie, "/api/mapping/root").await;
    assert_eq!(root_node["children"][0]["path"], "/second");
    assert_eq!(root_node["children"][1]["path"], "/first");
}

#[tokio::test]
async fn mapping_reorder_rejects_unknown_id() {
    let (root, app) = test_app("mapping-reorder-missing").await;
    let files_dir = root.path().join("files");
    tokio::fs::create_dir_all(&files_dir).await.unwrap();

    let cookie = login_cookie(&app).await;
    create_mapping_with_order(&app, &cookie, "/files", &files_dir, 10).await;

    let response = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/api/mapping/reorder",
            Some(&cookie),
            json!({
                "items": [
                    { "id": 404, "order": 1 }
                ]
            }),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = json_body(response).await;
    assert_eq!(body["code"], "NOT_FOUND");

    let after = get_json(&app, &cookie, "/api/mapping").await;
    assert_eq!(after[0]["mountPath"], "/files");
    assert_eq!(after[0]["order"], 10);
}

#[tokio::test]
async fn mapping_rejects_missing_local_folder_as_bad_request() {
    let (root, app) = test_app("mapping-missing-local-folder").await;
    let missing_dir = root.path().join("missing");

    let cookie = login_cookie(&app).await;
    let response = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/api/mapping",
            Some(&cookie),
            json!({
                "mountPath": "/missing",
                "folderPath": path_text(&missing_dir),
                "remark": "不存在的本地目录",
                "order": 0,
                "writable": true
            }),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert_eq!(body["code"], "BAD_REQUEST");
    assert_eq!(body["reason"], "MAPPING_FOLDER_PATH_NOT_FOUND");
    assert_eq!(body["params"]["path"], path_text(&missing_dir));
}

#[tokio::test]
async fn mapping_paths_do_not_expose_windows_extended_prefix() {
    let (root, app) = test_app("mapping-display-path").await;
    let files_dir = root.path().join("files");
    tokio::fs::create_dir_all(&files_dir).await.unwrap();

    let cookie = login_cookie(&app).await;
    create_mapping_with_order(&app, &cookie, "/files", &files_dir, 10).await;

    let persisted_text = tokio::fs::read_to_string(root.path().join("data/mappings.json"))
        .await
        .unwrap();
    let persisted: Value = serde_json::from_str(&persisted_text).unwrap();
    assert_no_windows_extended_prefix(&persisted[0]["folderPath"]);

    let mappings = get_json(&app, &cookie, "/api/mapping").await;
    assert_no_windows_extended_prefix(&mappings[0]["folderPath"]);

    let root_node = get_json(&app, &cookie, "/api/mapping/root").await;
    assert_no_windows_extended_prefix(&root_node["children"][0]["realPath"]);
}

async fn create_mapping_with_order(
    app: &Router,
    cookie: &str,
    mount_path: &str,
    path: &Path,
    order: i32,
) -> i64 {
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
                "order": order,
                "writable": true
            }),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    json_body(response).await.as_i64().unwrap()
}

fn assert_no_windows_extended_prefix(value: &Value) {
    let path = value.as_str().unwrap();
    assert!(
        !path.starts_with("\\\\?\\"),
        "不应向配置或接口暴露 Windows 扩展路径前缀: {path}"
    );
}
