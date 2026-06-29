use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::http::{HeaderMap, header::COOKIE};
use tokio::sync::RwLock;
use uuid::Uuid;

pub const SESSION_COOKIE: &str = "wfb_session";
const SESSION_MAX_AGE_SECONDS: u64 = 604_800;
const SESSION_TTL: Duration = Duration::from_secs(SESSION_MAX_AGE_SECONDS);
const LOGIN_FAILURE_LIMIT: u32 = 5;
const LOGIN_FAILURE_WINDOW: Duration = Duration::from_secs(10 * 60);
const LOGIN_FAILURE_COOLDOWN: Duration = Duration::from_secs(30);

#[derive(Clone, Default)]
pub struct AuthService {
    sessions: Arc<RwLock<HashMap<String, Instant>>>,
    login_failures: Arc<RwLock<HashMap<String, LoginFailure>>>,
}

#[derive(Debug, Clone)]
struct LoginFailure {
    attempts: u32,
    last_failed_at: Instant,
    blocked_until: Option<Instant>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoginThrottle {
    pub retry_after: Duration,
    pub attempts: u32,
}

impl AuthService {
    pub async fn create_session(&self) -> String {
        let token = Uuid::new_v4().to_string();
        let mut sessions = self.sessions.write().await;
        prune_expired_sessions(&mut sessions);
        sessions.insert(token.clone(), Instant::now() + SESSION_TTL);
        token
    }

    pub async fn remove_session(&self, token: &str) {
        self.sessions.write().await.remove(token);
    }

    pub async fn clear_sessions(&self) {
        self.sessions.write().await.clear();
    }

    pub async fn is_valid(&self, token: &str) -> bool {
        let mut sessions = self.sessions.write().await;
        prune_expired_sessions(&mut sessions);
        sessions.contains_key(token)
    }

    pub async fn count(&self) -> usize {
        let mut sessions = self.sessions.write().await;
        prune_expired_sessions(&mut sessions);
        sessions.len()
    }

    pub async fn login_cooldown(&self, key: &str) -> Option<LoginThrottle> {
        let mut failures = self.login_failures.write().await;
        prune_login_failures(&mut failures);
        login_throttle_for(failures.get(key), Instant::now())
    }

    pub async fn record_login_failure(&self, key: &str) -> Option<LoginThrottle> {
        let now = Instant::now();
        let mut failures = self.login_failures.write().await;
        prune_login_failures_at(&mut failures, now);
        let entry = failures.entry(key.to_string()).or_insert(LoginFailure {
            attempts: 0,
            last_failed_at: now,
            blocked_until: None,
        });
        entry.attempts = entry.attempts.saturating_add(1);
        entry.last_failed_at = now;
        if entry.attempts >= LOGIN_FAILURE_LIMIT {
            entry.blocked_until = Some(now + LOGIN_FAILURE_COOLDOWN);
        }
        login_throttle_for(Some(entry), now)
    }

    pub async fn clear_login_failures(&self, key: &str) {
        self.login_failures.write().await.remove(key);
    }
}

fn prune_expired_sessions(sessions: &mut HashMap<String, Instant>) {
    let now = Instant::now();
    sessions.retain(|_, expires_at| *expires_at > now);
}

fn login_throttle_for(failure: Option<&LoginFailure>, now: Instant) -> Option<LoginThrottle> {
    let failure = failure?;
    let blocked_until = failure.blocked_until?;
    (blocked_until > now).then(|| LoginThrottle {
        retry_after: blocked_until.duration_since(now),
        attempts: failure.attempts,
    })
}

fn prune_login_failures(failures: &mut HashMap<String, LoginFailure>) {
    prune_login_failures_at(failures, Instant::now());
}

fn prune_login_failures_at(failures: &mut HashMap<String, LoginFailure>, now: Instant) {
    failures.retain(|_, failure| {
        if failure
            .blocked_until
            .is_some_and(|blocked_until| blocked_until > now)
        {
            return true;
        }
        now.checked_duration_since(failure.last_failed_at)
            .is_some_and(|age| age < LOGIN_FAILURE_WINDOW)
    });
}

pub fn extract_session_token(headers: &HeaderMap) -> Option<String> {
    let cookie = headers.get(COOKIE)?.to_str().ok()?;
    cookie
        .split(';')
        .filter_map(|part| part.trim().split_once('='))
        .find_map(|(name, value)| (name == SESSION_COOKIE).then(|| value.to_string()))
}

pub fn session_cookie_value(token: &str) -> String {
    format!(
        "{SESSION_COOKIE}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={SESSION_MAX_AGE_SECONDS}"
    )
}

pub fn clear_session_cookie_value() -> String {
    format!("{SESSION_COOKIE}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn validates_created_session() {
        let auth = AuthService::default();
        let token = auth.create_session().await;

        assert!(auth.is_valid(&token).await);
        assert_eq!(auth.count().await, 1);
    }

    #[tokio::test]
    async fn prunes_expired_sessions_when_checking_validity() {
        let auth = AuthService::default();
        let expired_token = "expired-token".to_string();
        let active_token = "active-token".to_string();
        let now = Instant::now();
        {
            let mut sessions = auth.sessions.write().await;
            sessions.insert(
                expired_token.clone(),
                now.checked_sub(Duration::from_secs(1)).unwrap_or(now),
            );
            sessions.insert(active_token.clone(), now + SESSION_TTL);
        }

        assert!(!auth.is_valid(&expired_token).await);
        assert!(auth.is_valid(&active_token).await);
        assert_eq!(auth.count().await, 1);
    }

    #[test]
    fn session_cookie_uses_same_ttl_as_server_session() {
        let cookie = session_cookie_value("token");

        assert!(cookie.contains("Max-Age=604800"));
    }

    #[tokio::test]
    async fn login_failures_enter_short_cooldown_and_clear_after_success() {
        let auth = AuthService::default();
        let key = "192.168.1.20";

        for _ in 0..LOGIN_FAILURE_LIMIT - 1 {
            assert!(auth.record_login_failure(key).await.is_none());
        }

        let throttle = auth.record_login_failure(key).await.unwrap();
        assert_eq!(throttle.attempts, LOGIN_FAILURE_LIMIT);
        assert!(throttle.retry_after <= LOGIN_FAILURE_COOLDOWN);
        assert!(auth.login_cooldown(key).await.is_some());

        auth.clear_login_failures(key).await;
        assert!(auth.login_cooldown(key).await.is_none());
    }
}
