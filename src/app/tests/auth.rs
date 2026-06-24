use super::common::*;

#[tokio::test]
async fn session_reports_unconfigured_auth_before_setup() {
    let (_root, app) = test_app("auth-session-unconfigured").await;

    let response = app
        .oneshot(empty_request(Method::GET, "/api/auth/session"))
        .await
        .unwrap();
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["authenticated"], false);
    assert_eq!(body["authConfigured"], false);
}

#[tokio::test]
async fn login_requires_first_password_setup() {
    let (_root, app) = test_app("auth-login-before-setup").await;

    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/auth/login",
            None,
            json!({ "password": "test-password" }),
        ))
        .await
        .unwrap();
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "CONFLICT");
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("管理员密码尚未初始化")
    );
}

#[tokio::test]
async fn setup_password_writes_auth_hash_and_logs_in() {
    let (root, app) = test_app("auth-setup-flow").await;

    let response = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/api/auth/setup",
            None,
            json!({ "password": "test-password" }),
        ))
        .await
        .unwrap();
    let status = response.status();
    let cookie = response
        .headers()
        .get(SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["authenticated"], true);
    assert_eq!(body["authConfigured"], true);

    let auth_text = tokio::fs::read_to_string(root.path().join("data/auth.json"))
        .await
        .unwrap();
    assert!(auth_text.contains("adminPasswordHash"));
    assert!(!auth_text.contains("test-password"));
    assert!(!root.path().join("data/config.json").exists());

    let session = get_json(&app, &cookie, "/api/auth/session").await;
    assert_eq!(session["authenticated"], true);
    assert_eq!(session["authConfigured"], true);
}

#[tokio::test]
async fn setup_password_is_disabled_after_initialization() {
    let (_root, app) = test_app("auth-setup-once").await;
    let _cookie = login_cookie(&app).await;

    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/auth/setup",
            None,
            json!({ "password": "another-password" }),
        ))
        .await
        .unwrap();
    let status = response.status();
    let body = json_body(response).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "CONFLICT");
    assert!(body["message"].as_str().unwrap().contains("已经初始化"));
}
