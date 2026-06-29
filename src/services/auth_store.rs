use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use argon2::{
    Argon2, PasswordHasher, PasswordVerifier,
    password_hash::{PasswordHash, SaltString},
};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthSettings {
    #[serde(default)]
    pub admin_password_hash: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Clone)]
pub struct AuthStore {
    file_path: Arc<PathBuf>,
    settings: Arc<RwLock<AuthSettings>>,
}

impl AuthStore {
    pub async fn load(file_path: PathBuf) -> Result<Self, AppError> {
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let settings = match tokio::fs::read_to_string(&file_path).await {
            Ok(text) if text.trim().is_empty() => AuthSettings::default(),
            Ok(text) => serde_json::from_str(&text)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => AuthSettings::default(),
            Err(error) => return Err(error.into()),
        };

        Ok(Self {
            file_path: Arc::new(file_path),
            settings: Arc::new(RwLock::new(settings)),
        })
    }

    pub async fn has_admin_password(&self) -> bool {
        self.settings.read().await.admin_password_hash.is_some()
    }

    pub async fn verify_admin_password(&self, password: String) -> Result<bool, AppError> {
        let Some(hash) = self.settings.read().await.admin_password_hash.clone() else {
            return Err(AppError::conflict("管理员密码尚未初始化")
                .with_reason("ADMIN_PASSWORD_NOT_CONFIGURED"));
        };

        tokio::task::spawn_blocking(move || verify_password(hash, password)).await?
    }

    pub async fn initialize_admin_password(&self, password: String) -> Result<(), AppError> {
        let hash = hash_password(password).await?;
        let mut settings = self.settings.write().await;
        if settings.admin_password_hash.is_some() {
            return Err(AppError::conflict("管理员密码已经初始化，请直接登录")
                .with_reason("ADMIN_PASSWORD_ALREADY_CONFIGURED"));
        }
        save_new_password_hash(&self.file_path, &mut settings, hash).await
    }

    pub async fn set_admin_password(&self, password: String) -> Result<(), AppError> {
        let hash = hash_password(password).await?;
        let mut settings = self.settings.write().await;
        save_new_password_hash(&self.file_path, &mut settings, hash).await
    }
}

async fn save_new_password_hash(
    file_path: &Path,
    settings: &mut AuthSettings,
    hash: String,
) -> Result<(), AppError> {
    let mut next = settings.clone();
    next.admin_password_hash = Some(hash);
    next.updated_at = Some(current_time_text());
    save_settings(file_path, &next).await?;
    *settings = next;
    Ok(())
}

async fn save_settings(file_path: &Path, settings: &AuthSettings) -> Result<(), AppError> {
    if let Some(parent) = file_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let text = serde_json::to_vec_pretty(settings)?;
    let file_name = file_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("auth.json");
    let temp_path = file_path.with_file_name(format!(".{file_name}.{}.tmp", Uuid::new_v4()));
    tokio::fs::write(&temp_path, text).await?;

    match tokio::fs::rename(&temp_path, file_path).await {
        Ok(()) => Ok(()),
        Err(_error) if cfg!(windows) && file_path.exists() => {
            tokio::fs::remove_file(file_path).await?;
            if let Err(rename_error) = tokio::fs::rename(&temp_path, file_path).await {
                let _ = tokio::fs::remove_file(&temp_path).await;
                Err(rename_error.into())
            } else {
                Ok(())
            }
        }
        Err(error) => {
            let _ = tokio::fs::remove_file(&temp_path).await;
            Err(error.into())
        }
    }
}

async fn hash_password(password: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|error| AppError::internal(format!("密码哈希处理失败: {error}")))
    })
    .await?
}

fn verify_password(hash: String, password: String) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(&hash)
        .map_err(|error| AppError::internal(format!("管理员密码哈希无效: {error}")))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

fn current_time_text() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| OffsetDateTime::now_utc().unix_timestamp().to_string())
}

#[cfg(test)]
mod tests {
    use super::AuthStore;

    #[tokio::test]
    async fn auth_store_writes_hash_without_plaintext_password() {
        let dir = temp_dir("web-file-browser-auth-store-test");
        let file = dir.join("auth.json");
        let store = AuthStore::load(file.clone()).await.unwrap();

        store
            .set_admin_password("test-password".to_string())
            .await
            .unwrap();

        let text = tokio::fs::read_to_string(&file).await.unwrap();
        assert!(text.contains("adminPasswordHash"));
        assert!(!text.contains("test-password"));
        assert!(store.has_admin_password().await);
        assert!(
            store
                .verify_admin_password("test-password".to_string())
                .await
                .unwrap()
        );

        tokio::fs::remove_dir_all(dir).await.unwrap();
    }

    #[tokio::test]
    async fn initialize_admin_password_only_succeeds_once() {
        let dir = temp_dir("web-file-browser-auth-store-init-once-test");
        let file = dir.join("auth.json");
        let store = AuthStore::load(file).await.unwrap();

        let first = {
            let store = store.clone();
            tokio::spawn(async move {
                store
                    .initialize_admin_password("first-password".to_string())
                    .await
            })
        };
        let second = {
            let store = store.clone();
            tokio::spawn(async move {
                store
                    .initialize_admin_password("second-password".to_string())
                    .await
            })
        };
        let first = first.await.unwrap();
        let second = second.await.unwrap();

        assert_eq!(first.is_ok() as usize + second.is_ok() as usize, 1);
        let first_password_works = store
            .verify_admin_password("first-password".to_string())
            .await
            .unwrap();
        let second_password_works = store
            .verify_admin_password("second-password".to_string())
            .await
            .unwrap();
        assert_ne!(first_password_works, second_password_works);

        tokio::fs::remove_dir_all(dir).await.unwrap();
    }

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nonce}"))
    }
}
