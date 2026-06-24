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
