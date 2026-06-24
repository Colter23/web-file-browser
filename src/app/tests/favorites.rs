use super::common::*;

#[tokio::test]
async fn favorites_create_list_update_reorder_and_delete_work_through_api() {
    let (root, app) = test_app("favorites-api-flow").await;
    let files_dir = root.path().join("files");
    tokio::fs::create_dir_all(files_dir.join("archive"))
        .await
        .unwrap();

    let cookie = login_cookie(&app).await;
    create_mapping(&app, &cookie, "/docs", &files_dir, true).await;

    let response = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/api/favorites",
            Some(&cookie),
            json!({
                "path": "/docs/archive",
                "name": "归档",
                "order": 20
            }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let created = json_body(response).await;
    assert_eq!(created["path"], "/docs/archive");
    assert_eq!(created["name"], "归档");
    assert_eq!(created["missing"], Value::Null);
    let id = created["id"].as_str().unwrap().to_string();

    let response = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/api/favorites",
            Some(&cookie),
            json!({ "path": "/docs/archive" }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);

    let list = get_json(&app, &cookie, "/api/favorites").await;
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["path"], "/docs/archive");
    assert_eq!(list[0]["missing"], Value::Null);

    let response = app
        .clone()
        .oneshot(json_request(
            Method::PATCH,
            &format!("/api/favorites/{id}"),
            Some(&cookie),
            json!({
                "name": "资料归档",
                "order": 5
            }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let updated = json_body(response).await;
    assert_eq!(updated["name"], "资料归档");
    assert_eq!(updated["order"], 5);

    let response = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/api/favorites/reorder",
            Some(&cookie),
            json!({
                "items": [
                    { "id": id, "order": 30 }
                ]
            }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let persisted = tokio::fs::read_to_string(root.path().join("data/favorites.json"))
        .await
        .unwrap();
    assert!(persisted.contains("资料归档"));

    let response = app
        .clone()
        .oneshot(empty_request_with_cookie(
            Method::DELETE,
            &format!("/api/favorites/{id}"),
            &cookie,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let list = get_json(&app, &cookie, "/api/favorites").await;
    assert!(list.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn favorites_check_reports_missing_without_removing_item() {
    let (root, app) = test_app("favorites-missing-check-api").await;
    let files_dir = root.path().join("files");
    let target_dir = files_dir.join("archive");
    tokio::fs::create_dir_all(&target_dir).await.unwrap();

    let cookie = login_cookie(&app).await;
    create_mapping(&app, &cookie, "/docs", &files_dir, true).await;

    let response = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/api/favorites",
            Some(&cookie),
            json!({ "path": "/docs/archive" }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    tokio::fs::remove_dir_all(&target_dir).await.unwrap();

    let unchecked = get_json(&app, &cookie, "/api/favorites").await;
    assert_eq!(unchecked[0]["missing"], Value::Null);

    let checked = get_json(&app, &cookie, "/api/favorites?check=true").await;
    assert_eq!(checked[0]["path"], "/docs/archive");
    assert_eq!(checked[0]["missing"], true);

    let still_present = get_json(&app, &cookie, "/api/favorites").await;
    assert_eq!(still_present.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn favorites_reject_files_and_internal_trash_paths() {
    let (root, app) = test_app("favorites-reject-invalid-api").await;
    let files_dir = root.path().join("files");
    tokio::fs::create_dir_all(files_dir.join(".web-file-browser-trash"))
        .await
        .unwrap();
    tokio::fs::write(files_dir.join("note.txt"), b"note")
        .await
        .unwrap();

    let cookie = login_cookie(&app).await;
    create_mapping(&app, &cookie, "/docs", &files_dir, true).await;

    let response = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/api/favorites",
            Some(&cookie),
            json!({ "path": "/docs/note.txt" }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let response = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/api/favorites",
            Some(&cookie),
            json!({ "path": "/docs/.web-file-browser-trash" }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn favorite_keeps_working_after_mount_path_changes_when_mount_id_is_same() {
    let (root, app) = test_app("favorites-mount-rename-api").await;
    let files_dir = root.path().join("files");
    tokio::fs::create_dir_all(files_dir.join("archive"))
        .await
        .unwrap();

    let cookie = login_cookie(&app).await;
    let response = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/api/mapping",
            Some(&cookie),
            json!({
                "mountPath": "/docs",
                "folderPath": path_text(&files_dir),
                "remark": "测试挂载",
                "order": 0,
                "writable": true
            }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let mount_id = json_body(response).await.as_i64().unwrap();

    let response = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/api/favorites",
            Some(&cookie),
            json!({ "path": "/docs/archive" }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let response = app
        .clone()
        .oneshot(json_request(
            Method::PUT,
            &format!("/api/mapping/{mount_id}"),
            Some(&cookie),
            json!({
                "mountPath": "/资料",
                "folderPath": path_text(&files_dir),
                "remark": "测试挂载",
                "order": 0,
                "writable": true
            }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let checked = get_json(&app, &cookie, "/api/favorites?check=true").await;
    assert_eq!(checked[0]["mountId"], mount_id);
    assert_eq!(checked[0]["mountPath"], "/资料");
    assert_eq!(checked[0]["path"], "/资料/archive");
    assert_eq!(checked[0]["missing"], false);
}

#[tokio::test]
async fn favorites_require_authentication() {
    let (_root, app) = test_app("favorites-auth-api").await;

    let response = app
        .clone()
        .oneshot(empty_request(Method::GET, "/api/favorites"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
