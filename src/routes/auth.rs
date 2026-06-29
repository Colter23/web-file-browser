use axum::{
    Json, Router,
    extract::{Extension, Request, State},
    http::{HeaderMap, HeaderValue, header::SET_COOKIE},
    middleware::Next,
    response::Response,
    routing::{get, post},
};
use std::sync::Arc;

use crate::{
    app::AppState,
    error::AppError,
    models::{ChangePasswordRequest, LoginRequest, SessionResponse, SetupPasswordRequest},
    routes::RequestIp,
    services::auth::{
        LoginThrottle, clear_session_cookie_value, extract_session_token, session_cookie_value,
    },
};

pub fn public_auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/setup", post(setup_password))
        .route("/auth/session", get(session))
}

pub fn protected_auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/logout", post(logout))
        .route("/auth/password", post(change_password))
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
        Err(AppError::unauthorized("请先登录").with_reason("AUTH_REQUIRED"))
    }
}

async fn login(
    State(state): State<Arc<AppState>>,
    Extension(request_ip): Extension<RequestIp>,
    Json(request): Json<LoginRequest>,
) -> Result<(HeaderMap, Json<SessionResponse>), AppError> {
    if !state.auth_store.has_admin_password().await {
        return Err(AppError::conflict("管理员密码尚未初始化，请先完成首次设置")
            .with_reason("ADMIN_PASSWORD_NOT_CONFIGURED"));
    }
    if let Some(throttle) = state.auth.login_cooldown(&request_ip.0).await {
        return Err(login_cooldown_error(throttle));
    }

    if !state
        .auth_store
        .verify_admin_password(request.password)
        .await?
    {
        if let Some(throttle) = state.auth.record_login_failure(&request_ip.0).await {
            return Err(login_cooldown_error(throttle));
        }
        return Err(
            AppError::unauthorized("管理员密码不正确").with_reason("ADMIN_PASSWORD_INCORRECT")
        );
    }

    state.auth.clear_login_failures(&request_ip.0).await;
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

fn login_cooldown_error(throttle: LoginThrottle) -> AppError {
    let retry_after_seconds = throttle.retry_after.as_secs().max(1);
    AppError::too_many_requests("管理员密码错误次数过多，请稍后重试")
        .with_reason("LOGIN_FAILURE_COOLDOWN")
        .with_param("retryAfterSeconds", retry_after_seconds)
        .with_param("attempts", throttle.attempts)
}

async fn setup_password(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SetupPasswordRequest>,
) -> Result<(HeaderMap, Json<SessionResponse>), AppError> {
    validate_setup_password(&request)?;
    state
        .auth_store
        .initialize_admin_password(request.password)
        .await?;
    state.auth.clear_sessions().await;
    let token = state.auth.create_session().await;
    audit_ignore(&state, "setupPassword", None, None).await;

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

async fn change_password(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<(HeaderMap, Json<SessionResponse>), AppError> {
    validate_new_password(&request)?;

    if !state
        .auth_store
        .verify_admin_password(request.current_password)
        .await?
    {
        return Err(AppError::unauthorized("当前管理员密码不正确")
            .with_reason("CURRENT_PASSWORD_INCORRECT"));
    }

    state
        .auth_store
        .set_admin_password(request.new_password)
        .await?;
    state.auth.clear_sessions().await;
    let token = state.auth.create_session().await;
    audit_ignore(&state, "changePassword", None, None).await;

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

fn validate_new_password(request: &ChangePasswordRequest) -> Result<(), AppError> {
    if request.current_password.is_empty() {
        return Err(AppError::bad_request("当前密码不能为空")
            .with_reason("CURRENT_PASSWORD_EMPTY")
            .with_param("field", "currentPassword"));
    }
    if request.new_password.chars().count() < 8 {
        return Err(AppError::bad_request("新密码长度不能少于 8 个字符")
            .with_reason("PASSWORD_TOO_SHORT")
            .with_param("field", "newPassword")
            .with_param("minLength", 8));
    }
    if request.current_password == request.new_password {
        return Err(
            AppError::bad_request("新密码不能与当前密码相同").with_reason("PASSWORD_REUSED")
        );
    }
    Ok(())
}

fn validate_setup_password(request: &SetupPasswordRequest) -> Result<(), AppError> {
    if request.password.chars().count() < 8 {
        return Err(AppError::bad_request("管理员密码长度不能少于 8 个字符")
            .with_reason("PASSWORD_TOO_SHORT")
            .with_param("field", "password")
            .with_param("minLength", 8));
    }
    Ok(())
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
    let auth_configured = state.auth_store.has_admin_password().await;

    Json(SessionResponse {
        authenticated,
        auth_configured,
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        error::AppError,
        models::{ChangePasswordRequest, SetupPasswordRequest},
        routes::auth::{validate_new_password, validate_setup_password},
    };

    #[test]
    fn rejects_empty_current_password() {
        let error = validate_new_password(&ChangePasswordRequest {
            current_password: String::new(),
            new_password: "new-password".to_string(),
        })
        .unwrap_err();

        assert!(matches!(error, AppError::BadRequest(_)));
    }

    #[test]
    fn rejects_short_new_password() {
        let error = validate_new_password(&ChangePasswordRequest {
            current_password: "old-password".to_string(),
            new_password: "short".to_string(),
        })
        .unwrap_err();

        assert!(matches!(error, AppError::BadRequest(_)));
    }

    #[test]
    fn rejects_reused_password() {
        let error = validate_new_password(&ChangePasswordRequest {
            current_password: "same-password".to_string(),
            new_password: "same-password".to_string(),
        })
        .unwrap_err();

        assert!(matches!(error, AppError::BadRequest(_)));
    }

    #[test]
    fn rejects_short_setup_password() {
        let error = validate_setup_password(&SetupPasswordRequest {
            password: "short".to_string(),
        })
        .unwrap_err();

        assert!(matches!(error, AppError::BadRequest(_)));
    }
}
