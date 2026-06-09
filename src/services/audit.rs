use std::{path::PathBuf, sync::Arc};

use serde::Serialize;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt,
    sync::Mutex,
};

use crate::error::AppError;

#[derive(Clone)]
pub struct AuditService {
    file_path: Arc<PathBuf>,
    write_lock: Arc<Mutex<()>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AuditRecord<'a> {
    time: String,
    actor: &'a str,
    action: &'a str,
    path: Option<&'a str>,
    detail: Option<&'a str>,
}

impl AuditService {
    pub async fn load(file_path: PathBuf) -> Result<Self, AppError> {
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        Ok(Self {
            file_path: Arc::new(file_path),
            write_lock: Arc::new(Mutex::new(())),
        })
    }

    pub async fn record(
        &self,
        actor: &str,
        action: &str,
        path: Option<&str>,
        detail: Option<&str>,
    ) -> Result<(), AppError> {
        let _guard = self.write_lock.lock().await;
        let time = OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .map_err(|error| AppError::internal(format!("生成审计时间失败: {error}")))?;
        let record = AuditRecord {
            time,
            actor,
            action,
            path,
            detail,
        };
        let mut line = serde_json::to_vec(&record)?;
        line.push(b'\n');
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&*self.file_path)
            .await?;
        file.write_all(&line).await?;
        Ok(())
    }
}
