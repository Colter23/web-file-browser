use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration as StdDuration, Instant},
};

use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

use crate::{
    error::AppError,
    models::ConflictPolicy,
    services::{
        conflict,
        path_resolver::{
            MappingSnapshot, ResolvedParentPath, join_virtual_path, resolve_parent_for_child_sync,
        },
    },
};

const RETENTION_CHECK_INTERVAL: StdDuration = StdDuration::from_secs(300);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashRecord {
    pub id: String,
    pub original_virtual_path: String,
    pub original_real_path: String,
    pub trash_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    pub deleted_at: String,
    pub actor: String,
    pub kind: TrashEntryKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashRestoreResult {
    pub record: TrashRecord,
    pub restored_virtual_path: String,
    pub restored_real_path: String,
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
    retention_days: Option<u64>,
    max_bytes: Option<u64>,
    last_retention_check: Arc<RwLock<Option<Instant>>>,
    retention_cleanup_lock: Arc<Mutex<()>>,
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

        Ok(Self {
            root: Arc::new(root),
            index_file: Arc::new(index_file),
            records: Arc::new(RwLock::new(records)),
            retention_days,
            max_bytes,
            last_retention_check: Arc::new(RwLock::new(None)),
            retention_cleanup_lock: Arc::new(Mutex::new(())),
        })
    }

    pub async fn cleanup_retention_if_due(&self) -> Result<usize, AppError> {
        if !self.has_retention_policy() {
            return Ok(0);
        }

        if self.retention_check_is_recent().await {
            return Ok(0);
        }

        let _guard = self.retention_cleanup_lock.lock().await;
        if self.retention_check_is_recent().await {
            return Ok(0);
        }

        let removed = self.apply_retention().await?;
        *self.last_retention_check.write().await = Some(Instant::now());
        Ok(removed)
    }

    pub async fn cleanup_retention(&self) -> Result<usize, AppError> {
        if !self.has_retention_policy() {
            return Ok(0);
        }

        let _guard = self.retention_cleanup_lock.lock().await;
        let removed = self.apply_retention().await?;
        *self.last_retention_check.write().await = Some(Instant::now());
        Ok(removed)
    }

    pub async fn list(&self) -> Vec<TrashRecord> {
        let mut records = self.records.read().await.clone();
        records.sort_by(|left, right| right.deleted_at.cmp(&left.deleted_at));
        records
    }

    pub async fn count(&self) -> usize {
        self.records.read().await.len()
    }

    pub async fn restore(
        &self,
        snapshot: Arc<MappingSnapshot>,
        id: String,
        policy: ConflictPolicy,
    ) -> Result<TrashRestoreResult, AppError> {
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
            move || {
                let parent =
                    resolve_parent_for_child_sync(&snapshot, &record.original_virtual_path)?;
                restore_sync(record, parent, policy)
            }
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

    async fn apply_retention(&self) -> Result<usize, AppError> {
        let mut records = self.records.read().await.clone();
        records.sort_by(|left, right| left.deleted_at.cmp(&right.deleted_at));
        let purge_ids = select_retention_purge_ids(&records, self.retention_days, self.max_bytes)?;

        self.purge_records(&purge_ids).await
    }

    async fn purge_records(&self, ids: &[String]) -> Result<usize, AppError> {
        if ids.is_empty() {
            return Ok(0);
        }

        let id_set = ids.iter().cloned().collect::<HashSet<_>>();
        let records_to_purge = self
            .records
            .read()
            .await
            .iter()
            .filter(|record| id_set.contains(&record.id))
            .cloned()
            .collect::<Vec<_>>();
        if records_to_purge.is_empty() {
            return Ok(0);
        }

        tokio::task::spawn_blocking({
            let records_to_purge = records_to_purge.clone();
            move || {
                for record in &records_to_purge {
                    purge_record_sync(record)?;
                }
                Ok::<_, AppError>(())
            }
        })
        .await??;

        let mut records = self.records.write().await;
        let before = records.len();
        records.retain(|record| !id_set.contains(&record.id));
        let removed = before.saturating_sub(records.len());
        self.save_all(&records).await?;
        Ok(removed)
    }

    fn has_retention_policy(&self) -> bool {
        self.retention_days.is_some() || self.max_bytes.is_some()
    }

    async fn retention_check_is_recent(&self) -> bool {
        self.last_retention_check
            .read()
            .await
            .as_ref()
            .is_some_and(|last_checked| last_checked.elapsed() < RETENTION_CHECK_INTERVAL)
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

fn restore_sync(
    record: TrashRecord,
    parent: ResolvedParentPath,
    policy: ConflictPolicy,
) -> Result<TrashRestoreResult, AppError> {
    let target = conflict::resolve_child(
        &parent.parent_real_path,
        &parent.child_name,
        &parent.child_virtual_path,
        policy,
    )?;
    if target.existed {
        match record.kind {
            TrashEntryKind::File => conflict::ensure_file_overwrite_allowed(&target)?,
            TrashEntryKind::Folder => return Err(AppError::conflict("不支持覆盖恢复目录")),
        }
        conflict::replace_file_sync(Path::new(&record.trash_path), &target.path)?;
    } else {
        if target.path.exists() {
            return Err(AppError::conflict(format!(
                "路径已存在: {}",
                join_virtual_path(&parent.parent_virtual_path, &target.name)
            )));
        }
        move_payload_with_copy_fallback(Path::new(&record.trash_path), &target.path, &record.kind)?;
    }
    let restored_virtual_path = join_virtual_path(&parent.parent_virtual_path, &target.name);
    let restored_real_path = target.path.to_string_lossy().to_string();
    remove_record_dir(&record)?;
    Ok(TrashRestoreResult {
        record,
        restored_virtual_path,
        restored_real_path,
    })
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
    if let Some(size_bytes) = record.size_bytes {
        return Ok(size_bytes);
    }

    dir_size(
        Path::new(&record.trash_path)
            .parent()
            .ok_or_else(|| AppError::bad_request("回收站记录路径无效"))?,
    )
    .map_err(AppError::from)
}

fn select_retention_purge_ids(
    records: &[TrashRecord],
    retention_days: Option<u64>,
    max_bytes: Option<u64>,
) -> Result<Vec<String>, AppError> {
    if retention_days.is_none() && max_bytes.is_none() {
        return Ok(Vec::new());
    }

    let mut purge_ids = Vec::new();
    let mut purge_set = HashSet::new();

    if let Some(retention_days) = retention_days {
        let retention_days = retention_days.min(i64::MAX as u64) as i64;
        let cutoff = OffsetDateTime::now_utc().saturating_sub(time::Duration::days(retention_days));
        for record in records {
            if parse_deleted_at(&record.deleted_at)
                .map(|deleted_at| deleted_at < cutoff)
                .unwrap_or(false)
                && purge_set.insert(record.id.clone())
            {
                purge_ids.push(record.id.clone());
            }
        }
    }

    if let Some(max_bytes) = max_bytes {
        let mut total = 0_u64;
        let mut retained_sizes = Vec::new();
        for record in records {
            if purge_set.contains(&record.id) {
                continue;
            }

            let size = record_size(record)?;
            total = total.saturating_add(size);
            retained_sizes.push((record.id.clone(), size));
        }

        for (id, size) in retained_sizes {
            if total <= max_bytes {
                break;
            }

            if purge_set.insert(id.clone()) {
                total = total.saturating_sub(size);
                purge_ids.push(id);
            }
        }
    }

    Ok(purge_ids)
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
    let size_bytes = matches!(kind, TrashEntryKind::File).then_some(metadata.len());

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
        size_bytes,
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

fn move_payload_with_copy_fallback(
    source: &Path,
    target: &Path,
    kind: &TrashEntryKind,
) -> Result<(), AppError> {
    if target.exists() {
        return Err(AppError::conflict(format!(
            "恢复目标已存在: {}",
            target.display()
        )));
    }
    match fs::rename(source, target) {
        Ok(()) => Ok(()),
        Err(rename_error) => copy_then_remove(source, target, kind).map_err(|copy_error| {
            AppError::internal(format!("恢复回收站条目失败: {rename_error}; {copy_error}"))
        }),
    }
}

fn copy_dir_recursively(source: &Path, target: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_source = entry.path();
        let entry_target = target.join(entry.file_name());
        let metadata = entry_source.symlink_metadata()?;
        let file_type = metadata.file_type();
        if file_type.is_symlink() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("不支持移动符号链接到回收站: {}", entry_source.display()),
            ));
        }
        if metadata.is_dir() {
            copy_dir_recursively(&entry_source, &entry_target)?;
        } else if metadata.is_file() {
            fs::copy(entry_source, entry_target)?;
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("不支持移动特殊文件到回收站: {}", entry_source.display()),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        TrashEntryKind, TrashRecord, TrashService, copy_then_remove,
        move_payload_with_copy_fallback, move_to_trash_sync, select_retention_purge_ids,
    };
    use crate::{
        error::AppError,
        models::{ConflictPolicy, PathMapping},
        services::path_resolver::MappingSnapshot,
    };
    use std::{
        fs,
        path::{Path, PathBuf},
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
        assert_eq!(record.size_bytes, Some(5));

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn copy_fallback_moves_file_and_folder_payload() {
        let temp = temp_dir("web-file-browser-trash-copy-fallback-test");
        let source_dir = temp.join("source");
        let target_dir = temp.join("target");
        fs::create_dir_all(&source_dir).unwrap();
        fs::create_dir_all(&target_dir).unwrap();

        let source_file = source_dir.join("hello.txt");
        let target_file = target_dir.join("hello.txt");
        fs::write(&source_file, "hello").unwrap();

        copy_then_remove(&source_file, &target_file, &TrashEntryKind::File).unwrap();

        assert!(!source_file.exists());
        assert_eq!(fs::read_to_string(&target_file).unwrap(), "hello");

        let folder_source = source_dir.join("folder");
        let nested_source = folder_source.join("nested");
        let folder_target = target_dir.join("folder");
        fs::create_dir_all(&nested_source).unwrap();
        fs::write(nested_source.join("item.txt"), "nested").unwrap();

        copy_then_remove(&folder_source, &folder_target, &TrashEntryKind::Folder).unwrap();

        assert!(!folder_source.exists());
        assert_eq!(
            fs::read_to_string(folder_target.join("nested/item.txt")).unwrap(),
            "nested"
        );

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn restore_payload_rejects_existing_target_before_copy_fallback() {
        let temp = temp_dir("web-file-browser-trash-restore-existing-target-test");
        fs::create_dir_all(&temp).unwrap();
        let source = temp.join("source.txt");
        let target = temp.join("target.txt");
        fs::write(&source, "source").unwrap();
        fs::write(&target, "target").unwrap();

        let result = move_payload_with_copy_fallback(&source, &target, &TrashEntryKind::File);

        assert!(result.is_err());
        assert_eq!(fs::read_to_string(&source).unwrap(), "source");
        assert_eq!(fs::read_to_string(&target).unwrap(), "target");

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn retention_uses_recorded_file_size_without_stat() {
        let old = test_record("old", "2000-01-01T00:00:00Z", Some(5));
        let current = test_record("current", "2000-01-02T00:00:00Z", Some(7));

        let purge_ids =
            select_retention_purge_ids(&[old.clone(), current.clone()], None, Some(7)).unwrap();

        assert_eq!(purge_ids, vec![old.id]);
    }

    #[tokio::test]
    async fn cleanup_retention_removes_records_added_after_startup() {
        let temp = temp_dir("web-file-browser-trash-retention-test");
        let source_dir = temp.join("source");
        let trash_dir = temp.join("trash");
        fs::create_dir_all(&source_dir).unwrap();

        let service = TrashService::load(trash_dir.clone(), Some(1), None)
            .await
            .unwrap();

        let old_source = source_dir.join("old.txt");
        let current_source = source_dir.join("current.txt");
        fs::write(&old_source, "old").unwrap();
        fs::write(&current_source, "current").unwrap();

        let mut old = move_to_trash_sync(
            trash_dir.clone(),
            old_source.clone(),
            "/repo/old.txt".to_string(),
            old_source.to_string_lossy().to_string(),
            "admin".to_string(),
        )
        .unwrap();
        old.deleted_at = "2000-01-01T00:00:00Z".to_string();
        let current = move_to_trash_sync(
            trash_dir,
            current_source.clone(),
            "/repo/current.txt".to_string(),
            current_source.to_string_lossy().to_string(),
            "admin".to_string(),
        )
        .unwrap();

        let snapshot = {
            let mut records = service.records.write().await;
            records.push(old.clone());
            records.push(current.clone());
            records.clone()
        };
        service.save_all(&snapshot).await.unwrap();

        let removed = service.cleanup_retention().await.unwrap();
        let records = service.list().await;

        assert_eq!(removed, 1);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id, current.id);
        assert!(!Path::new(&old.trash_path).exists());
        assert!(Path::new(&current.trash_path).exists());

        fs::remove_dir_all(temp).unwrap();
    }

    #[tokio::test]
    async fn load_does_not_apply_retention_until_requested() {
        let temp = temp_dir("web-file-browser-trash-deferred-retention-test");
        let source_dir = temp.join("source");
        let trash_dir = temp.join("trash");
        fs::create_dir_all(&source_dir).unwrap();
        fs::create_dir_all(&trash_dir).unwrap();

        let old_source = source_dir.join("old.txt");
        fs::write(&old_source, "old").unwrap();
        let mut old = move_to_trash_sync(
            trash_dir.clone(),
            old_source.clone(),
            "/repo/old.txt".to_string(),
            old_source.to_string_lossy().to_string(),
            "admin".to_string(),
        )
        .unwrap();
        old.deleted_at = "2000-01-01T00:00:00Z".to_string();
        fs::write(
            trash_dir.join("index.json"),
            serde_json::to_vec_pretty(&vec![old.clone()]).unwrap(),
        )
        .unwrap();

        let service = TrashService::load(trash_dir, Some(1), None).await.unwrap();

        assert_eq!(service.list().await.len(), 1);
        assert!(Path::new(&old.trash_path).exists());
        assert_eq!(service.cleanup_retention_if_due().await.unwrap(), 1);
        assert!(service.list().await.is_empty());
        assert!(!Path::new(&old.trash_path).exists());

        fs::remove_dir_all(temp).unwrap();
    }

    #[tokio::test]
    async fn cleanup_retention_if_due_is_throttled_after_first_run() {
        let temp = temp_dir("web-file-browser-trash-retention-throttle-test");
        let source_dir = temp.join("source");
        let trash_dir = temp.join("trash");
        fs::create_dir_all(&source_dir).unwrap();

        let service = TrashService::load(trash_dir.clone(), Some(1), None)
            .await
            .unwrap();
        let first_source = source_dir.join("old-1.txt");
        let second_source = source_dir.join("old-2.txt");
        fs::write(&first_source, "old 1").unwrap();
        fs::write(&second_source, "old 2").unwrap();

        let mut first = move_to_trash_sync(
            trash_dir.clone(),
            first_source.clone(),
            "/repo/old-1.txt".to_string(),
            first_source.to_string_lossy().to_string(),
            "admin".to_string(),
        )
        .unwrap();
        first.deleted_at = "2000-01-01T00:00:00Z".to_string();

        let snapshot = {
            let mut records = service.records.write().await;
            records.push(first.clone());
            records.clone()
        };
        service.save_all(&snapshot).await.unwrap();

        assert_eq!(service.cleanup_retention_if_due().await.unwrap(), 1);
        assert!(service.list().await.is_empty());

        let mut second = move_to_trash_sync(
            trash_dir,
            second_source.clone(),
            "/repo/old-2.txt".to_string(),
            second_source.to_string_lossy().to_string(),
            "admin".to_string(),
        )
        .unwrap();
        second.deleted_at = "2000-01-01T00:00:00Z".to_string();

        let snapshot = {
            let mut records = service.records.write().await;
            records.push(second.clone());
            records.clone()
        };
        service.save_all(&snapshot).await.unwrap();

        assert_eq!(service.cleanup_retention_if_due().await.unwrap(), 0);
        assert_eq!(service.list().await.len(), 1);

        assert_eq!(service.cleanup_retention().await.unwrap(), 1);
        assert!(service.list().await.is_empty());

        fs::remove_dir_all(temp).unwrap();
    }

    #[tokio::test]
    async fn restores_with_auto_rename_when_original_path_exists() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp =
            std::env::temp_dir().join(format!("web-file-browser-trash-restore-test-{nonce}"));
        let source_dir = temp.join("source");
        let trash_dir = temp.join("trash");
        fs::create_dir_all(&source_dir).unwrap();
        let service = TrashService::load(trash_dir.clone(), None, None)
            .await
            .unwrap();
        let source = source_dir.join("hello.txt");
        fs::write(&source, "deleted").unwrap();

        let record = move_to_trash_sync(
            trash_dir,
            source.clone(),
            "/repo/hello.txt".to_string(),
            source.to_string_lossy().to_string(),
            "admin".to_string(),
        )
        .unwrap();
        let snapshot = {
            let mut records = service.records.write().await;
            records.push(record.clone());
            records.clone()
        };
        service.save_all(&snapshot).await.unwrap();
        fs::write(&source, "current").unwrap();

        let restored = service
            .restore(
                snapshot_for_mount(&source_dir, true).await,
                record.id,
                ConflictPolicy::AutoRename,
            )
            .await
            .unwrap();

        assert_eq!(fs::read_to_string(&source).unwrap(), "current");
        assert_eq!(
            fs::read_to_string(source_dir.join("hello (1).txt")).unwrap(),
            "deleted"
        );
        assert_eq!(restored.restored_virtual_path, "/repo/hello (1).txt");
        assert!(service.list().await.is_empty());
        fs::remove_dir_all(temp).unwrap();
    }

    #[tokio::test]
    async fn restore_rejects_readonly_current_mount() {
        let temp = temp_dir("web-file-browser-trash-readonly-restore-test");
        let source_dir = temp.join("source");
        let trash_dir = temp.join("trash");
        fs::create_dir_all(&source_dir).unwrap();
        let service = TrashService::load(trash_dir.clone(), None, None)
            .await
            .unwrap();
        let source = source_dir.join("hello.txt");
        fs::write(&source, "deleted").unwrap();

        let record = move_to_trash_sync(
            trash_dir,
            source.clone(),
            "/repo/hello.txt".to_string(),
            source.to_string_lossy().to_string(),
            "admin".to_string(),
        )
        .unwrap();
        let snapshot = {
            let mut records = service.records.write().await;
            records.push(record.clone());
            records.clone()
        };
        service.save_all(&snapshot).await.unwrap();

        let error = service
            .restore(
                snapshot_for_mount(&source_dir, false).await,
                record.id.clone(),
                ConflictPolicy::AutoRename,
            )
            .await
            .unwrap_err();

        assert!(matches!(error, AppError::Forbidden(_)));
        assert_eq!(service.list().await.len(), 1);
        assert!(Path::new(&record.trash_path).exists());
        assert!(!source.exists());
        fs::remove_dir_all(temp).unwrap();
    }

    fn temp_dir(prefix: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nonce}"))
    }

    fn test_record(id: &str, deleted_at: &str, size_bytes: Option<u64>) -> TrashRecord {
        TrashRecord {
            id: id.to_string(),
            original_virtual_path: format!("/repo/{id}.txt"),
            original_real_path: format!("missing/{id}.txt"),
            trash_path: format!("missing-trash/{id}/payload.txt"),
            size_bytes,
            deleted_at: deleted_at.to_string(),
            actor: "admin".to_string(),
            kind: TrashEntryKind::File,
        }
    }

    async fn snapshot_for_mount(path: &Path, writable: bool) -> std::sync::Arc<MappingSnapshot> {
        MappingSnapshot::build(vec![PathMapping {
            id: Some(1),
            mount_path: "/repo".to_string(),
            folder_path: path.to_string_lossy().to_string(),
            remark: Some(String::new()),
            order: Some(0),
            writable,
        }])
        .await
        .unwrap()
    }
}
