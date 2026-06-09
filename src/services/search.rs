use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    thread,
    time::Duration,
};

use tokio::sync::RwLock;

use crate::{
    error::AppError,
    models::{EntryKind, IndexStatus, PathMapping, SearchResponse, SearchResult},
    services::path_resolver::{MappingSnapshot, join_virtual_path},
};

#[derive(Clone)]
pub struct SearchService {
    enabled: bool,
    entries: Arc<RwLock<Vec<SearchResult>>>,
    status: Arc<RwLock<IndexStatus>>,
}

impl SearchService {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            entries: Arc::new(RwLock::new(Vec::new())),
            status: Arc::new(RwLock::new(IndexStatus {
                enabled,
                state: if enabled { "idle" } else { "disabled" }.to_string(),
                indexed_entries: 0,
                last_started_at: None,
                last_finished_at: None,
                last_error: None,
            })),
        }
    }

    pub async fn rebuild(
        &self,
        snapshot: Arc<MappingSnapshot>,
        scan_delay_ms: u64,
    ) -> Result<(), AppError> {
        if !self.enabled {
            return Err(AppError::bad_request("搜索索引未启用"));
        }
        {
            let mut status = self.status.write().await;
            if status.state == "scanning" {
                return Err(AppError::conflict("索引正在重建"));
            }
            status.state = "scanning".to_string();
            status.last_started_at = Some(now_epoch_string());
            status.last_error = None;
        }

        let mappings = snapshot.mappings.clone();
        let result =
            tokio::task::spawn_blocking(move || scan_mappings(&mappings, scan_delay_ms)).await?;
        match result {
            Ok(entries) => {
                let indexed_entries = entries.len();
                *self.entries.write().await = entries;
                let mut status = self.status.write().await;
                status.state = "idle".to_string();
                status.indexed_entries = indexed_entries;
                status.last_finished_at = Some(now_epoch_string());
                Ok(())
            }
            Err(error) => {
                let mut status = self.status.write().await;
                status.state = "failed".to_string();
                status.last_error = Some(error.to_string());
                Err(error)
            }
        }
    }

    pub async fn search(
        &self,
        q: Option<String>,
        mount: Option<String>,
        entry_type: Option<String>,
        offset: usize,
        limit: usize,
    ) -> SearchResponse {
        let q = q.map(|value| value.to_lowercase()).unwrap_or_default();
        let items = self.entries.read().await;
        let filtered = items
            .iter()
            .filter(|item| {
                (q.is_empty()
                    || item.name.to_lowercase().contains(&q)
                    || item.path.to_lowercase().contains(&q)
                    || item.extension.to_lowercase().contains(&q))
                    && mount
                        .as_ref()
                        .map(|mount| &item.mount_path == mount)
                        .unwrap_or(true)
                    && entry_type
                        .as_ref()
                        .map(|entry_type| &item.entry_type == entry_type)
                        .unwrap_or(true)
            })
            .cloned()
            .collect::<Vec<_>>();
        let total = filtered.len();
        let items = filtered
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect::<Vec<_>>();
        SearchResponse {
            items,
            total,
            offset,
            limit,
        }
    }

    pub async fn recent(&self, limit: usize) -> Vec<SearchResult> {
        let mut items = self.entries.read().await.clone();
        items.sort_by(|left, right| right.modified.cmp(&left.modified));
        items.into_iter().take(limit).collect()
    }

    pub async fn status(&self) -> IndexStatus {
        self.status.read().await.clone()
    }

    pub async fn count(&self) -> usize {
        self.entries.read().await.len()
    }
}

fn scan_mappings(
    mappings: &[PathMapping],
    scan_delay_ms: u64,
) -> Result<Vec<SearchResult>, AppError> {
    let mut entries = Vec::new();
    for mapping in mappings {
        let root = PathBuf::from(&mapping.folder_path);
        scan_dir(mapping, &root, &root, scan_delay_ms, &mut entries)?;
    }
    Ok(entries)
}

fn scan_dir(
    mapping: &PathMapping,
    root: &Path,
    dir: &Path,
    scan_delay_ms: u64,
    entries: &mut Vec<SearchResult>,
) -> Result<(), AppError> {
    if scan_delay_ms > 0 {
        thread::sleep(Duration::from_millis(scan_delay_ms));
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        let kind = if file_type.is_dir() {
            EntryKind::Folder
        } else {
            EntryKind::File
        };
        entries.push(search_result(mapping, root, &path, kind)?);
        if file_type.is_dir() {
            scan_dir(mapping, root, &path, scan_delay_ms, entries)?;
        }
    }
    Ok(())
}

fn search_result(
    mapping: &PathMapping,
    root: &Path,
    path: &Path,
    kind: EntryKind,
) -> Result<SearchResult, AppError> {
    let metadata = fs::metadata(path)?;
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default();
    let relative = path.strip_prefix(root).unwrap_or(path);
    let mut virtual_path = mapping.mount_path.clone();
    for component in relative.components() {
        let part = component.as_os_str().to_string_lossy();
        if part.is_empty() {
            continue;
        }
        virtual_path = join_virtual_path(&virtual_path, &part);
    }
    let extension = path
        .extension()
        .map(|extension| extension.to_string_lossy().to_string())
        .unwrap_or_default();
    Ok(SearchResult {
        name,
        path: virtual_path,
        extension,
        modified: metadata
            .modified()
            .ok()
            .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs().to_string())
            .unwrap_or_default(),
        size: metadata.len(),
        entry_type: kind.as_str().to_string(),
        mount_path: mapping.mount_path.clone(),
    })
}

fn now_epoch_string() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_default()
}
