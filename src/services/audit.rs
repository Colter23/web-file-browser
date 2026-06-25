use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
};

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
    state: Arc<Mutex<AuditState>>,
    max_bytes: Arc<AtomicU64>,
    retention_files: Arc<AtomicUsize>,
}

#[derive(Debug)]
struct AuditState {
    current_bytes: u64,
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
    pub async fn load(
        file_path: PathBuf,
        max_bytes: Option<u64>,
        retention_files: usize,
    ) -> Result<Self, AppError> {
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        let current_bytes = match fs::metadata(&file_path).await {
            Ok(metadata) => metadata.len(),
            Err(error) if error.kind() == ErrorKind::NotFound => 0,
            Err(error) => return Err(error.into()),
        };
        Ok(Self {
            file_path: Arc::new(file_path),
            state: Arc::new(Mutex::new(AuditState { current_bytes })),
            max_bytes: Arc::new(AtomicU64::new(option_to_atomic(max_bytes))),
            retention_files: Arc::new(AtomicUsize::new(retention_files)),
        })
    }

    pub fn update_policy(&self, max_bytes: Option<u64>, retention_files: usize) {
        self.max_bytes
            .store(option_to_atomic(max_bytes), Ordering::Relaxed);
        self.retention_files
            .store(retention_files, Ordering::Relaxed);
    }

    pub async fn record(
        &self,
        actor: &str,
        action: &str,
        path: Option<&str>,
        detail: Option<&str>,
    ) -> Result<(), AppError> {
        let mut state = self.state.lock().await;
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
        self.rotate_if_needed(&mut state, line.len() as u64).await?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&*self.file_path)
            .await?;
        file.write_all(&line).await?;
        state.current_bytes = state.current_bytes.saturating_add(line.len() as u64);
        Ok(())
    }

    pub async fn cleanup_rotated(&self) -> Result<usize, AppError> {
        cleanup_rotated_audit_files(&self.file_path, self.retention_files()).await
    }

    async fn rotate_if_needed(
        &self,
        state: &mut AuditState,
        incoming_bytes: u64,
    ) -> Result<(), AppError> {
        let Some(max_bytes) = self.max_bytes() else {
            return Ok(());
        };
        if state.current_bytes == 0
            || state.current_bytes.saturating_add(incoming_bytes) <= max_bytes
        {
            return Ok(());
        }

        let rotated_path = rotated_audit_path(&self.file_path)?;
        match fs::rename(&*self.file_path, rotated_path).await {
            Ok(()) => {
                state.current_bytes = 0;
                if let Err(error) =
                    cleanup_rotated_audit_files(&self.file_path, self.retention_files()).await
                {
                    tracing::warn!("清理旧审计日志失败: {error}");
                }
                Ok(())
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {
                state.current_bytes = 0;
                Ok(())
            }
            Err(error) => Err(error.into()),
        }
    }

    fn max_bytes(&self) -> Option<u64> {
        atomic_to_option(self.max_bytes.load(Ordering::Relaxed))
    }

    fn retention_files(&self) -> usize {
        self.retention_files.load(Ordering::Relaxed)
    }
}

async fn cleanup_rotated_audit_files(
    file_path: &Path,
    retention_files: usize,
) -> Result<usize, AppError> {
    let Some(parent) = file_path.parent() else {
        return Ok(0);
    };
    let mut entries = Vec::new();
    let mut read_dir = fs::read_dir(parent).await?;

    while let Some(entry) = read_dir.next_entry().await? {
        let file_type = entry.file_type().await?;
        if !file_type.is_file() {
            continue;
        }

        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        let Some(timestamp) = rotated_audit_timestamp(file_path, &file_name)? else {
            continue;
        };
        entries.push((timestamp, entry.path()));
    }

    if entries.len() <= retention_files {
        return Ok(0);
    }

    entries.sort_by_key(|entry| std::cmp::Reverse(entry.0));
    let mut removed = 0_usize;
    for (_, path) in entries.into_iter().skip(retention_files) {
        match fs::remove_file(path).await {
            Ok(()) => removed += 1,
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }

    Ok(removed)
}

fn rotated_audit_path(file_path: &Path) -> Result<PathBuf, AppError> {
    let parent = file_path.parent();
    let file_name = file_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| AppError::bad_request("审计日志路径无效"))?;
    let timestamp = OffsetDateTime::now_utc().unix_timestamp_nanos();
    let rotated_name = match file_name.rsplit_once('.') {
        Some((stem, extension)) if !stem.is_empty() && !extension.is_empty() => {
            format!("{stem}.{timestamp}.{extension}")
        }
        _ => format!("{file_name}.{timestamp}"),
    };

    Ok(parent
        .map(|path| path.join(&rotated_name))
        .unwrap_or_else(|| PathBuf::from(rotated_name)))
}

fn option_to_atomic(value: Option<u64>) -> u64 {
    value.unwrap_or(0)
}

fn atomic_to_option(value: u64) -> Option<u64> {
    if value == 0 { None } else { Some(value) }
}

fn rotated_audit_timestamp(file_path: &Path, candidate: &str) -> Result<Option<i128>, AppError> {
    let file_name = file_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| AppError::bad_request("审计日志路径无效"))?;
    let (prefix, suffix) = match file_name.rsplit_once('.') {
        Some((stem, extension)) if !stem.is_empty() && !extension.is_empty() => {
            (format!("{stem}."), format!(".{extension}"))
        }
        _ => (format!("{file_name}."), String::new()),
    };

    if !candidate.starts_with(&prefix) || !candidate.ends_with(&suffix) {
        return Ok(None);
    }
    if candidate.len() <= prefix.len().saturating_add(suffix.len()) {
        return Ok(None);
    }

    let timestamp = &candidate[prefix.len()..candidate.len() - suffix.len()];
    if timestamp.is_empty() {
        return Ok(None);
    }

    Ok(timestamp.parse().ok())
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::*;

    #[tokio::test]
    async fn rotates_audit_file_when_size_limit_is_reached() {
        let dir = temp_dir("web-file-browser-audit-rotate-test");
        let audit_path = dir.join("audit.jsonl");
        let service = AuditService::load(audit_path.clone(), Some(180), 8)
            .await
            .unwrap();

        service
            .record("admin", "first.large.action", Some("/a"), Some("x"))
            .await
            .unwrap();
        service
            .record("admin", "second.large.action", Some("/b"), Some("x"))
            .await
            .unwrap();

        let rotated_count = fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                let file_name = entry.file_name();
                let file_name = file_name.to_string_lossy();
                file_name.starts_with("audit.") && file_name != "audit.jsonl"
            })
            .count();
        let current = fs::read_to_string(&audit_path).unwrap();

        assert_eq!(rotated_count, 1);
        assert!(current.contains("second.large.action"));

        let _ = fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn zero_audit_limit_disables_rotation() {
        let dir = temp_dir("web-file-browser-audit-no-rotate-test");
        let audit_path = dir.join("audit.jsonl");
        let service = AuditService::load(audit_path.clone(), None, 8)
            .await
            .unwrap();

        for index in 0..3 {
            service
                .record(
                    "admin",
                    "audit.test",
                    Some("/file"),
                    Some(&index.to_string()),
                )
                .await
                .unwrap();
        }

        let rotated_count = fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                let file_name = entry.file_name();
                let file_name = file_name.to_string_lossy();
                file_name.starts_with("audit.") && file_name != "audit.jsonl"
            })
            .count();

        assert_eq!(rotated_count, 0);

        let _ = fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn load_existing_size_before_rotating() {
        let dir = temp_dir("web-file-browser-audit-existing-size-test");
        let audit_path = dir.join("audit.jsonl");
        fs::write(&audit_path, "existing audit line\n").unwrap();
        let service = AuditService::load(audit_path.clone(), Some(20), 8)
            .await
            .unwrap();

        service
            .record("admin", "new.action", Some("/file"), None)
            .await
            .unwrap();

        let rotated_entry = fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .find(|entry| {
                let file_name = entry.file_name();
                let file_name = file_name.to_string_lossy();
                file_name.starts_with("audit.") && file_name != "audit.jsonl"
            })
            .expect("应该生成轮转后的审计日志");
        let rotated = fs::read_to_string(rotated_entry.path()).unwrap();
        let current = fs::read_to_string(&audit_path).unwrap();

        assert!(rotated.contains("existing audit line"));
        assert!(current.contains("new.action"));
        assert!(!current.contains("existing audit line"));

        let _ = fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn keeps_only_latest_rotated_audit_files() {
        let dir = temp_dir("web-file-browser-audit-retention-test");
        let audit_path = dir.join("audit.jsonl");
        for timestamp in [10, 20, 30] {
            fs::write(
                dir.join(format!("audit.{timestamp}.jsonl")),
                format!("old {timestamp}\n"),
            )
            .unwrap();
        }

        let removed = cleanup_rotated_audit_files(&audit_path, 2).await.unwrap();

        assert_eq!(removed, 1);
        assert!(!dir.join("audit.10.jsonl").exists());
        assert!(dir.join("audit.20.jsonl").exists());
        assert!(dir.join("audit.30.jsonl").exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn zero_retention_removes_rotated_audit_files_after_rotation() {
        let dir = temp_dir("web-file-browser-audit-zero-retention-test");
        let audit_path = dir.join("audit.jsonl");
        let service = AuditService::load(audit_path.clone(), Some(1), 0)
            .await
            .unwrap();

        service.record("admin", "first", None, None).await.unwrap();
        service.record("admin", "second", None, None).await.unwrap();

        let rotated_count = fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                let file_name = entry.file_name();
                let file_name = file_name.to_string_lossy();
                file_name.starts_with("audit.") && file_name != "audit.jsonl"
            })
            .count();

        assert_eq!(rotated_count, 0);
        assert!(fs::read_to_string(&audit_path).unwrap().contains("second"));

        let _ = fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn explicit_cleanup_returns_removed_count() {
        let dir = temp_dir("web-file-browser-audit-explicit-cleanup-test");
        let audit_path = dir.join("audit.jsonl");
        for timestamp in [10, 20, 30] {
            fs::write(
                dir.join(format!("audit.{timestamp}.jsonl")),
                format!("old {timestamp}\n"),
            )
            .unwrap();
        }
        let service = AuditService::load(audit_path, Some(1024), 1).await.unwrap();

        let removed = service.cleanup_rotated().await.unwrap();

        assert_eq!(removed, 2);
        assert!(!dir.join("audit.10.jsonl").exists());
        assert!(!dir.join("audit.20.jsonl").exists());
        assert!(dir.join("audit.30.jsonl").exists());

        let _ = fs::remove_dir_all(dir);
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("{name}-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }
}
