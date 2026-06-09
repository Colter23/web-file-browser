use std::{collections::HashSet, sync::Arc};

use axum::http::{HeaderMap, header::COOKIE};
use tokio::sync::RwLock;
use uuid::Uuid;

pub const SESSION_COOKIE: &str = "wfb_session";

#[derive(Clone, Default)]
pub struct AuthService {
    sessions: Arc<RwLock<HashSet<String>>>,
}

impl AuthService {
    pub async fn create_session(&self) -> String {
        let token = Uuid::new_v4().to_string();
        self.sessions.write().await.insert(token.clone());
        token
    }

    pub async fn remove_session(&self, token: &str) {
        self.sessions.write().await.remove(token);
    }

    pub async fn is_valid(&self, token: &str) -> bool {
        self.sessions.read().await.contains(token)
    }

    pub async fn count(&self) -> usize {
        self.sessions.read().await.len()
    }
}

pub fn extract_session_token(headers: &HeaderMap) -> Option<String> {
    let cookie = headers.get(COOKIE)?.to_str().ok()?;
    cookie
        .split(';')
        .filter_map(|part| part.trim().split_once('='))
        .find_map(|(name, value)| (name == SESSION_COOKIE).then(|| value.to_string()))
}

pub fn session_cookie_value(token: &str) -> String {
    format!("{SESSION_COOKIE}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age=604800")
}

pub fn clear_session_cookie_value() -> String {
    format!("{SESSION_COOKIE}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0")
}
