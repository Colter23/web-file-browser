use std::{path::PathBuf, sync::Arc};

use argon2::{
    Argon2, PasswordHasher, PasswordVerifier,
    password_hash::{PasswordHash, SaltString},
};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::error::AppError;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default)]
    pub admin_password_hash: Option<String>,
}

#[derive(Clone)]
pub struct SettingsStore {
    file_path: Arc<PathBuf>,
    settings: Arc<RwLock<AppSettings>>,
}

impl SettingsStore {
    pub async fn load(
        file_path: PathBuf,
        initial_admin_password: Option<String>,
    ) -> Result<Self, AppError> {
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut settings = match tokio::fs::read_to_string(&file_path).await {
            Ok(text) if text.trim().is_empty() => AppSettings::default(),
            Ok(text) => serde_json::from_str(&text)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => AppSettings::default(),
            Err(error) => return Err(error.into()),
        };

        if settings.admin_password_hash.is_none()
            && let Some(password) = initial_admin_password.filter(|password| !password.is_empty())
        {
            settings.admin_password_hash = Some(hash_password(password).await?);
            save_settings(&file_path, &settings).await?;
        }

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
            return Err(AppError::conflict("管理员密码尚未初始化"));
        };

        tokio::task::spawn_blocking(move || verify_password(hash, password)).await?
    }

    #[allow(dead_code)]
    pub async fn set_admin_password(&self, password: String) -> Result<(), AppError> {
        let hash = hash_password(password).await?;
        let mut settings = self.settings.write().await;
        settings.admin_password_hash = Some(hash);
        save_settings(&self.file_path, &settings).await
    }
}

async fn save_settings(file_path: &PathBuf, settings: &AppSettings) -> Result<(), AppError> {
    if let Some(parent) = file_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let text = serde_json::to_vec_pretty(settings)?;
    tokio::fs::write(file_path, text).await?;
    Ok(())
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
