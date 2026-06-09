use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashRecord {
    pub id: String,
    pub original_virtual_path: String,
    pub original_real_path: String,
    pub trash_path: String,
    pub deleted_at: String,
    pub actor: String,
    pub kind: TrashEntryKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TrashEntryKind {
    File,
    Folder,
}

#[derive(Clone)]
pub struct TrashService {
    root: Arc<PathBuf>,
    index_file: Arc<PathBuf>,
    records: Arc<RwLock<Vec<TrashRecord>>>,
}

impl TrashService {
    pub async fn load(
        root: PathBuf,
        retention_days: Option<u64>,
        max_bytes: Option<u64>,
    ) -> Result<Self, AppError> {
        tokio::fs::create_dir_all(&root).await?;
        let index_file = root.join("index.json");
        let records = match tokio::fs::read_to_string(&index_file).await {
            Ok(text) if text.trim().is_empty() => Vec::new(),
            Ok(text) => serde_json::from_str(&text)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(error) => return Err(error.into()),
        };

        let service = Self {
            root: Arc::new(root),
            index_file: Arc::new(index_file),
            records: Arc::new(RwLock::new(records)),
        };
        service.apply_retention(retention_days, max_bytes).await?;
        Ok(service)
    }

    pub async fn list(&self) -> Vec<TrashRecord> {
        let mut records = self.records.read().await.clone();
        records.sort_by(|left, right| right.deleted_at.cmp(&left.deleted_at));
        records
    }

    pub async fn count(&self) -> usize {
        self.records.read().await.len()
    }

    pub async fn restore(&self, id: String) -> Result<TrashRecord, AppError> {
        let record = self
            .records
            .read()
            .await
            .iter()
            .find(|record| record.id == id)
            .cloned()
            .ok_or_else(|| AppError::not_found(format!("查无此回收站记录: {id}")))?;
        let restored = tokio::task::spawn_blocking({
            let record = record.clone();
            move || restore_sync(record)
        })
        .await??;
        self.remove_record(&id).await?;
        Ok(restored)
    }

    pub async fn purge(&self, id: String) -> Result<(), AppError> {
        let record = self
            .records
            .read()
            .await
            .iter()
            .find(|record| record.id == id)
            .cloned()
            .ok_or_else(|| AppError::not_found(format!("查无此回收站记录: {id}")))?;
        tokio::task::spawn_blocking(move || purge_record_sync(&record)).await??;
        self.remove_record(&id).await
    }

    pub async fn empty(&self) -> Result<usize, AppError> {
        let records = self.records.read().await.clone();
        let count = records.len();
        tokio::task::spawn_blocking(move || {
            for record in &records {
                purge_record_sync(record)?;
            }
            Ok::<_, AppError>(())
        })
        .await??;
        let mut current = self.records.write().await;
        current.clear();
        self.save_all(&current).await?;
        Ok(count)
    }

    pub async fn move_to_trash(
        &self,
        source: PathBuf,
        original_virtual_path: String,
        original_real_path: String,
        actor: String,
    ) -> Result<TrashRecord, AppError> {
        let root = (*self.root).clone();
        let record = tokio::task::spawn_blocking(move || {
            move_to_trash_sync(
                root,
                source,
                original_virtual_path,
                original_real_path,
                actor,
            )
        })
        .await??;

        let mut records = self.records.write().await;
        records.push(record.clone());
        self.save_all(&records).await?;
        Ok(record)
    }

    async fn remove_record(&self, id: &str) -> Result<(), AppError> {
        let mut records = self.records.write().await;
        records.retain(|record| record.id != id);
        self.save_all(&records).await
    }

    async fn apply_retention(
        &self,
        retention_days: Option<u64>,
        max_bytes: Option<u64>,
    ) -> Result<(), AppError> {
        if retention_days.is_none() && max_bytes.is_none() {
            return Ok(());
        }

        let mut records = self.records.read().await.clone();
        records.sort_by(|left, right| left.deleted_at.cmp(&right.deleted_at));
        let mut purge_ids = Vec::new();

        if let Some(retention_days) = retention_days {
            let cutoff = OffsetDateTime::now_utc()
                .saturating_sub(time::Duration::days(retention_days as i64));
            for record in &records {
                if parse_deleted_at(&record.deleted_at)
                    .map(|deleted_at| deleted_at < cutoff)
                    .unwrap_or(false)
                {
                    purge_ids.push(record.id.clone());
                }
            }
        }

        if let Some(max_bytes) = max_bytes {
            let mut total = records
                .iter()
                .map(record_size)
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .sum::<u64>();
            for record in &records {
                if total <= max_bytes {
                    break;
                }
                if !purge_ids.contains(&record.id) {
                    total = total.saturating_sub(record_size(record)?);
                    purge_ids.push(record.id.clone());
                }
            }
        }

        if purge_ids.is_empty() {
            return Ok(());
        }

        for id in purge_ids {
            let _ = self.purge(id).await;
        }
        Ok(())
    }

    async fn save_all(&self, records: &[TrashRecord]) -> Result<(), AppError> {
        if let Some(parent) = self.index_file.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let text = serde_json::to_vec_pretty(records)?;
        tokio::fs::write(&*self.index_file, text).await?;
        Ok(())
    }
}

fn restore_sync(record: TrashRecord) -> Result<TrashRecord, AppError> {
    let target = PathBuf::from(&record.original_real_path);
    if target.exists() {
        return Err(AppError::conflict(format!(
            "原路径已存在，无法恢复: {}",
            record.original_virtual_path
        )));
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(&record.trash_path, &target)?;
    remove_record_dir(&record)?;
    Ok(record)
}

fn purge_record_sync(record: &TrashRecord) -> Result<(), AppError> {
    remove_record_dir(record)
}

fn remove_record_dir(record: &TrashRecord) -> Result<(), AppError> {
    let payload = Path::new(&record.trash_path);
    let record_dir = payload
        .parent()
        .ok_or_else(|| AppError::bad_request("回收站记录路径无效"))?;
    if record_dir.exists() {
        fs::remove_dir_all(record_dir)?;
    }
    Ok(())
}

fn record_size(record: &TrashRecord) -> Result<u64, AppError> {
    dir_size(
        Path::new(&record.trash_path)
            .parent()
            .ok_or_else(|| AppError::bad_request("回收站记录路径无效"))?,
    )
    .map_err(AppError::from)
}

fn dir_size(path: &Path) -> Result<u64, std::io::Error> {
    let metadata = fs::metadata(path)?;
    if metadata.is_file() {
        return Ok(metadata.len());
    }
    let mut size = 0;
    for entry in fs::read_dir(path)? {
        size += dir_size(&entry?.path())?;
    }
    Ok(size)
}

fn parse_deleted_at(value: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(value, &Rfc3339).ok()
}

fn move_to_trash_sync(
    root: PathBuf,
    source: PathBuf,
    original_virtual_path: String,
    original_real_path: String,
    actor: String,
) -> Result<TrashRecord, AppError> {
    let metadata = fs::metadata(&source)
        .map_err(|_| AppError::not_found(format!("查无此路径: {original_virtual_path}")))?;
    let kind = if metadata.is_dir() {
        TrashEntryKind::Folder
    } else {
        TrashEntryKind::File
    };

    fs::create_dir_all(&root)?;
    let id = Uuid::new_v4().to_string();
    let entry_dir = root.join(&id);
    fs::create_dir_all(&entry_dir)?;

    let name = source
        .file_name()
        .ok_or_else(|| AppError::bad_request("不能删除挂载根路径"))?;
    let payload_path = entry_dir.join(name);

    if let Err(rename_error) = fs::rename(&source, &payload_path) {
        copy_then_remove(&source, &payload_path, &kind)
            .map_err(|copy_error| AppError::internal(format!("{rename_error}; {copy_error}")))?;
    }

    let deleted_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| AppError::internal(format!("生成删除时间失败: {error}")))?;
    let record = TrashRecord {
        id,
        original_virtual_path,
        original_real_path,
        trash_path: payload_path.to_string_lossy().to_string(),
        deleted_at,
        actor,
        kind,
    };

    let metadata_text = serde_json::to_vec_pretty(&record)?;
    fs::write(entry_dir.join("metadata.json"), metadata_text)?;

    Ok(record)
}

fn copy_then_remove(
    source: &Path,
    target: &Path,
    kind: &TrashEntryKind,
) -> Result<(), std::io::Error> {
    match kind {
        TrashEntryKind::File => {
            fs::copy(source, target)?;
            fs::remove_file(source)?;
        }
        TrashEntryKind::Folder => {
            copy_dir_recursively(source, target)?;
            fs::remove_dir_all(source)?;
        }
    }
    Ok(())
}

fn copy_dir_recursively(source: &Path, target: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_source = entry.path();
        let entry_target = target.join(entry.file_name());
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            copy_dir_recursively(&entry_source, &entry_target)?;
        } else {
            fs::copy(entry_source, entry_target)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::move_to_trash_sync;
    use std::{
        fs,
        path::Path,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn moves_file_to_unique_trash_path() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!("web-file-browser-trash-test-{nonce}"));
        let source_dir = temp.join("source");
        let trash_dir = temp.join("trash");
        fs::create_dir_all(&source_dir).unwrap();
        let source = source_dir.join("hello.txt");
        fs::write(&source, "hello").unwrap();

        let record = move_to_trash_sync(
            trash_dir.clone(),
            source.clone(),
            "/repo/hello.txt".to_string(),
            source.to_string_lossy().to_string(),
            "admin".to_string(),
        )
        .unwrap();

        assert!(!source.exists());
        assert!(Path::new(&record.trash_path).exists());
        assert!(trash_dir.join(&record.id).join("metadata.json").exists());

        fs::remove_dir_all(temp).unwrap();
    }
}
