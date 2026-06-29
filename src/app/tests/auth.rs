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

#[tokio::test]
async fn public_auth_routes_are_limited_by_ip_concurrency() {
    let (_root, app) = test_app_with_config("auth-ip-limit-api", |config| {
        config.max_ip_concurrency = 1;
        config.trust_proxy_headers = true;
    })
    .await;

    let pending_body = Body::from_stream(stream::pending::<Result<Bytes, std::io::Error>>());
    let first_request = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header("x-forwarded-for", "10.0.0.10")
        .header(CONTENT_TYPE, "application/json")
        .body(pending_body)
        .unwrap();
    let first_login = tokio::spawn({
        let app = app.clone();
        async move { app.oneshot(first_request).await }
    });

    let mut last_status = None;
    let mut last_body = Value::Null;
    for _ in 0..100 {
        let second_request = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/login")
            .header("x-forwarded-for", "10.0.0.10")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(
                serde_json::to_vec(&json!({ "password": "test-password" })).unwrap(),
            ))
            .unwrap();
        let response = app.clone().oneshot(second_request).await.unwrap();
        let status = response.status();
        let body = json_body(response).await;
        if status == StatusCode::TOO_MANY_REQUESTS {
            assert_eq!(body["code"], "TOO_MANY_REQUESTS");
            assert_eq!(body["reason"], "IP_CONCURRENCY_LIMITED");
            first_login.abort();
            assert!(first_login.await.unwrap_err().is_cancelled());
            return;
        }
        last_status = Some(status);
        last_body = body;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    first_login.abort();
    assert!(first_login.await.unwrap_err().is_cancelled());
    panic!("公共认证接口未触发 IP 并发限制，最后状态: {last_status:?}, body: {last_body}");
}
