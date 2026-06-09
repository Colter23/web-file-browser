use axum::{
    Json, Router,
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, header::SET_COOKIE},
    middleware::Next,
    response::Response,
    routing::{get, post},
};
use std::sync::Arc;

use crate::{
    app::AppState,
    error::AppError,
    models::{LoginRequest, SessionResponse},
    services::auth::{clear_session_cookie_value, extract_session_token, session_cookie_value},
};

pub fn public_auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/session", get(session))
}

pub fn protected_auth_routes() -> Router<Arc<AppState>> {
    Router::new().route("/auth/logout", post(logout))
}

pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let authenticated = match extract_session_token(request.headers()) {
        Some(token) => state.auth.is_valid(&token).await,
        None => false,
    };

    if authenticated {
        Ok(next.run(request).await)
    } else {
        Err(AppError::unauthorized("请先登录"))
    }
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LoginRequest>,
) -> Result<(HeaderMap, Json<SessionResponse>), AppError> {
    if !state.settings.has_admin_password().await {
        return Err(AppError::conflict(
            "管理员密码尚未初始化，请设置 WEB_FILE_BROWSER_ADMIN_PASSWORD 后重启服务",
        ));
    }

    if !state
        .settings
        .verify_admin_password(request.password)
        .await?
    {
        return Err(AppError::unauthorized("管理员密码不正确"));
    }

    let token = state.auth.create_session().await;
    audit_ignore(&state, "login", None, None).await;
    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        HeaderValue::from_str(&session_cookie_value(&token))
            .map_err(|error| AppError::internal(format!("生成会话 Cookie 失败: {error}")))?,
    );

    Ok((
        headers,
        Json(SessionResponse {
            authenticated: true,
            auth_configured: true,
        }),
    ))
}

async fn logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> (HeaderMap, Json<SessionResponse>) {
    if let Some(token) = extract_session_token(&headers) {
        state.auth.remove_session(&token).await;
    }
    audit_ignore(&state, "logout", None, None).await;

    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        SET_COOKIE,
        HeaderValue::from_str(&clear_session_cookie_value())
            .unwrap_or_else(|_| HeaderValue::from_static("wfb_session=; Path=/; Max-Age=0")),
    );

    (
        response_headers,
        Json(SessionResponse {
            authenticated: false,
            auth_configured: true,
        }),
    )
}

async fn audit_ignore(state: &AppState, action: &str, path: Option<&str>, detail: Option<&str>) {
    if let Err(error) = state.audit.record("admin", action, path, detail).await {
        tracing::warn!("写入审计日志失败: {error}");
    }
}

async fn session(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Json<SessionResponse> {
    let authenticated = match extract_session_token(&headers) {
        Some(token) => state.auth.is_valid(&token).await,
        None => false,
    };
    let auth_configured = state.settings.has_admin_password().await;

    Json(SessionResponse {
        authenticated,
        auth_configured,
    })
}
