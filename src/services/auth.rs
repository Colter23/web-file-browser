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

#[derive(Clone, Default)]
pub struct AuthService {
    sessions: Arc<RwLock<HashMap<String, Instant>>>,
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
}

fn prune_expired_sessions(sessions: &mut HashMap<String, Instant>) {
    let now = Instant::now();
    sessions.retain(|_, expires_at| *expires_at > now);
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
}
