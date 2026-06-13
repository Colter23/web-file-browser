use std::{
    cmp::{Ordering as CmpOrdering, Reverse},
    collections::BinaryHeap,
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::Duration,
};

use tokio::sync::RwLock;

use crate::{
    error::AppError,
    models::{EntryKind, IndexStatus, PathMapping, SearchResponse, SearchResult},
    services::path_resolver::{
        MappingSnapshot, join_virtual_path, resolve_existing_sync, split_virtual_path,
        virtual_path_from_parts,
    },
};

#[derive(Clone)]
pub struct SearchService {
    enabled: bool,
    entries: Arc<RwLock<Vec<IndexedEntry>>>,
    status: Arc<RwLock<IndexStatus>>,
    rebuild_token: Arc<AtomicU64>,
}

#[derive(Debug, Clone)]
struct IndexedEntry {
    result: SearchResult,
    normalized_path: String,
    modified_epoch: u64,
}

#[derive(Debug, Eq, PartialEq)]
struct RecentCandidate {
    modified_epoch: u64,
    index: usize,
}

impl Ord for RecentCandidate {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        self.modified_epoch
            .cmp(&other.modified_epoch)
            .then_with(|| other.index.cmp(&self.index))
    }
}

impl PartialOrd for RecentCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

impl IndexedEntry {
    fn new(result: SearchResult) -> Self {
        let normalized_path = result.path.to_lowercase();
        let modified_epoch = parse_modified_epoch(&result.modified);
        Self {
            result,
            normalized_path,
            modified_epoch,
        }
    }

    fn refresh_normalized_path(&mut self) {
        self.normalized_path = self.result.path.to_lowercase();
    }
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
            rebuild_token: Arc::new(AtomicU64::new(0)),
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
        let token = {
            let mut status = self.status.write().await;
            if status.state == "scanning" {
                return Err(AppError::conflict("索引正在重建"));
            }
            status.state = "scanning".to_string();
            status.last_started_at = Some(now_epoch_string());
            status.last_error = None;
            status.last_finished_at = None;
            self.rebuild_token.fetch_add(1, Ordering::SeqCst) + 1
        };

        let mappings = snapshot.mappings.clone();
        let rebuild_token = self.rebuild_token.clone();
        let result = tokio::task::spawn_blocking(move || {
            scan_mappings(&mappings, scan_delay_ms, &rebuild_token, token)
        })
        .await?;
        match result {
            Ok(entries) => {
                if self.rebuild_token.load(Ordering::SeqCst) != token {
                    self.mark_rebuild_cancelled().await;
                    return Err(AppError::conflict("索引重建已取消"));
                }
                let indexed_entries = entries.len();
                *self.entries.write().await = entries;
                let mut status = self.status.write().await;
                status.state = "idle".to_string();
                status.indexed_entries = indexed_entries;
                status.last_finished_at = Some(now_epoch_string());
                Ok(())
            }
            Err(ScanError::Cancelled) => {
                self.mark_rebuild_cancelled().await;
                Err(AppError::conflict("索引重建已取消"))
            }
            Err(error) => {
                let mut status = self.status.write().await;
                status.state = "failed".to_string();
                status.last_finished_at = Some(now_epoch_string());
                status.last_error = Some(error.to_string());
                Err(error.into())
            }
        }
    }

    pub async fn cancel_rebuild(&self) -> Result<(), AppError> {
        if !self.enabled {
            return Err(AppError::bad_request("搜索索引未启用"));
        }
        let is_scanning = self.status.read().await.state == "scanning";
        if !is_scanning {
            return Err(AppError::conflict("当前没有正在重建的索引"));
        }
        self.rebuild_token.fetch_add(1, Ordering::SeqCst);
        self.mark_rebuild_cancelled().await;
        Ok(())
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
        let mut total = 0;
        let mut page_items = Vec::with_capacity(limit);
        for item in items
            .iter()
            .filter(|item| matches_search(item, &q, mount.as_deref(), entry_type.as_deref()))
        {
            if total >= offset && page_items.len() < limit {
                page_items.push(item.result.clone());
            }
            total += 1;
        }
        SearchResponse {
            items: page_items,
            total,
            offset,
            limit,
        }
    }

    pub async fn recent(&self, limit: usize) -> Vec<SearchResult> {
        if limit == 0 {
            return Vec::new();
        }
        let items = self.entries.read().await;
        let mut candidates = BinaryHeap::with_capacity(limit);
        for (index, item) in items.iter().enumerate() {
            let candidate = RecentCandidate {
                modified_epoch: item.modified_epoch,
                index,
            };
            if candidates.len() < limit {
                candidates.push(Reverse(candidate));
                continue;
            }
            if candidates
                .peek()
                .map(|oldest| candidate > oldest.0)
                .unwrap_or(true)
            {
                candidates.pop();
                candidates.push(Reverse(candidate));
            }
        }

        let mut candidates = candidates
            .into_iter()
            .map(|Reverse(candidate)| candidate)
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| right.cmp(left));
        candidates
            .into_iter()
            .map(|candidate| items[candidate.index].result.clone())
            .collect()
    }

    pub async fn upsert_virtual_path(
        &self,
        snapshot: Arc<MappingSnapshot>,
        virtual_path: String,
    ) -> Result<(), AppError> {
        if !self.enabled {
            return Ok(());
        }
        let item = tokio::task::spawn_blocking(move || {
            let resolved = resolve_existing_sync(&snapshot, &virtual_path)?;
            let root = PathBuf::from(&resolved.mapping.folder_path);
            search_result_from_path(&resolved.mapping, &root, &resolved.real_path)
        })
        .await??;

        let indexed_entries = {
            let mut entries = self.entries.write().await;
            remove_path_prefix(&mut entries, &item.path);
            entries.push(IndexedEntry::new(item));
            entries.len()
        };
        self.refresh_indexed_entries(indexed_entries).await;
        Ok(())
    }

    pub async fn remove_virtual_path(&self, virtual_path: &str) -> Result<(), AppError> {
        if !self.enabled {
            return Ok(());
        }
        let virtual_path = normalize_virtual_path(virtual_path)?;
        let indexed_entries = {
            let mut entries = self.entries.write().await;
            remove_path_prefix(&mut entries, &virtual_path);
            entries.len()
        };
        self.refresh_indexed_entries(indexed_entries).await;
        Ok(())
    }

    pub async fn move_virtual_path(&self, old_path: &str, new_path: &str) -> Result<(), AppError> {
        if !self.enabled {
            return Ok(());
        }
        let old_path = normalize_virtual_path(old_path)?;
        let new_path = normalize_virtual_path(new_path)?;
        let old_prefix = child_prefix(&old_path);
        let mut entries = self.entries.write().await;
        for item in entries.iter_mut() {
            if item.result.path == old_path {
                item.result.path = new_path.clone();
                refresh_name_and_extension(&mut item.result);
                item.refresh_normalized_path();
            } else if let Some(relative) = item.result.path.strip_prefix(&old_prefix) {
                item.result.path = join_virtual_path(&new_path, relative);
                refresh_name_and_extension(&mut item.result);
                item.refresh_normalized_path();
            }
        }
        Ok(())
    }

    pub async fn remove_mount(&self, mount_path: &str) -> Result<(), AppError> {
        if !self.enabled {
            return Ok(());
        }
        let mount_path = normalize_virtual_path(mount_path)?;
        let indexed_entries = {
            let mut entries = self.entries.write().await;
            entries.retain(|item| item.result.mount_path != mount_path);
            entries.len()
        };
        self.refresh_indexed_entries(indexed_entries).await;
        Ok(())
    }

    pub async fn status(&self) -> IndexStatus {
        self.status.read().await.clone()
    }

    async fn refresh_indexed_entries(&self, indexed_entries: usize) {
        let mut status = self.status.write().await;
        status.indexed_entries = indexed_entries;
    }

    async fn mark_rebuild_cancelled(&self) {
        let mut status = self.status.write().await;
        status.state = "cancelled".to_string();
        status.last_finished_at = Some(now_epoch_string());
        status.last_error = Some("索引重建已取消".to_string());
    }
}

fn matches_search(
    item: &IndexedEntry,
    q: &str,
    mount: Option<&str>,
    entry_type: Option<&str>,
) -> bool {
    let query_matches = q.is_empty() || item.normalized_path.contains(q);
    let mount_matches = mount
        .map(|mount| item.result.mount_path == mount)
        .unwrap_or(true);
    let type_matches = entry_type
        .map(|entry_type| item.result.entry_type == entry_type)
        .unwrap_or(true);
    query_matches && mount_matches && type_matches
}

#[derive(Debug)]
enum ScanError {
    App(AppError),
    Cancelled,
}

impl From<AppError> for ScanError {
    fn from(error: AppError) -> Self {
        Self::App(error)
    }
}

impl From<std::io::Error> for ScanError {
    fn from(error: std::io::Error) -> Self {
        Self::App(error.into())
    }
}

impl From<ScanError> for AppError {
    fn from(error: ScanError) -> Self {
        match error {
            ScanError::App(error) => error,
            ScanError::Cancelled => AppError::conflict("索引重建已取消"),
        }
    }
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::App(error) => write!(formatter, "{error}"),
            ScanError::Cancelled => write!(formatter, "索引重建已取消"),
        }
    }
}

fn scan_mappings(
    mappings: &[PathMapping],
    scan_delay_ms: u64,
    rebuild_token: &AtomicU64,
    expected_token: u64,
) -> Result<Vec<IndexedEntry>, ScanError> {
    let mut entries = Vec::new();
    for mapping in mappings {
        ensure_scan_not_cancelled(rebuild_token, expected_token)?;
        let root = PathBuf::from(&mapping.folder_path);
        scan_dir(
            mapping,
            &root,
            &root,
            scan_delay_ms,
            rebuild_token,
            expected_token,
            &mut entries,
        )?;
    }
    Ok(entries)
}

fn scan_dir(
    mapping: &PathMapping,
    root: &Path,
    dir: &Path,
    scan_delay_ms: u64,
    rebuild_token: &AtomicU64,
    expected_token: u64,
    entries: &mut Vec<IndexedEntry>,
) -> Result<(), ScanError> {
    ensure_scan_not_cancelled(rebuild_token, expected_token)?;
    if scan_delay_ms > 0 {
        thread::sleep(Duration::from_millis(scan_delay_ms));
    }
    ensure_scan_not_cancelled(rebuild_token, expected_token)?;
    for entry in fs::read_dir(dir)? {
        ensure_scan_not_cancelled(rebuild_token, expected_token)?;
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        let kind = if file_type.is_dir() {
            EntryKind::Folder
        } else {
            EntryKind::File
        };
        entries.push(IndexedEntry::new(search_result(
            mapping, root, &path, kind,
        )?));
        if file_type.is_dir() {
            scan_dir(
                mapping,
                root,
                &path,
                scan_delay_ms,
                rebuild_token,
                expected_token,
                entries,
            )?;
        }
    }
    Ok(())
}

fn ensure_scan_not_cancelled(
    rebuild_token: &AtomicU64,
    expected_token: u64,
) -> Result<(), ScanError> {
    if rebuild_token.load(Ordering::SeqCst) == expected_token {
        Ok(())
    } else {
        Err(ScanError::Cancelled)
    }
}

fn search_result(
    mapping: &PathMapping,
    root: &Path,
    path: &Path,
    kind: EntryKind,
) -> Result<SearchResult, AppError> {
    let metadata = fs::metadata(path)?;
    search_result_with_metadata(mapping, root, path, kind, &metadata)
}

fn search_result_from_path(
    mapping: &PathMapping,
    root: &Path,
    path: &Path,
) -> Result<SearchResult, AppError> {
    let metadata = fs::metadata(path)?;
    let kind = if metadata.is_dir() {
        EntryKind::Folder
    } else {
        EntryKind::File
    };
    search_result_with_metadata(mapping, root, path, kind, &metadata)
}

fn search_result_with_metadata(
    mapping: &PathMapping,
    root: &Path,
    path: &Path,
    kind: EntryKind,
    metadata: &fs::Metadata,
) -> Result<SearchResult, AppError> {
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

fn normalize_virtual_path(path: &str) -> Result<String, AppError> {
    let parts = split_virtual_path(path)?;
    Ok(virtual_path_from_parts(&parts))
}

fn child_prefix(path: &str) -> String {
    if path == "/" {
        "/".to_string()
    } else {
        format!("{}/", path.trim_end_matches('/'))
    }
}

fn remove_path_prefix(entries: &mut Vec<IndexedEntry>, path: &str) {
    let prefix = child_prefix(path);
    entries.retain(|item| item.result.path != path && !item.result.path.starts_with(&prefix));
}

fn refresh_name_and_extension(item: &mut SearchResult) {
    item.name = item
        .path
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .to_string();
    item.extension = Path::new(&item.name)
        .extension()
        .map(|extension| extension.to_string_lossy().to_string())
        .unwrap_or_default();
}

fn parse_modified_epoch(modified: &str) -> u64 {
    modified.parse::<u64>().unwrap_or(0)
}

fn now_epoch_string() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn disabled_incremental_update_is_noop() {
        let service = SearchService::new(false);

        service.remove_virtual_path("/missing").await.unwrap();

        assert_eq!(service.status().await.indexed_entries, 0);
    }

    #[tokio::test]
    async fn upserts_and_removes_single_path() {
        let (snapshot, temp) = snapshot_with_files("search-upsert", &[("a.txt", "hello")]).await;
        let service = SearchService::new(true);

        service
            .upsert_virtual_path(snapshot, "/repo/a.txt".to_string())
            .await
            .unwrap();
        let found = service
            .search(Some("a".to_string()), None, None, 0, 10)
            .await;
        assert_eq!(found.total, 1);
        assert_eq!(found.items[0].path, "/repo/a.txt");

        service.remove_virtual_path("/repo/a.txt").await.unwrap();
        let empty = service
            .search(Some("a".to_string()), None, None, 0, 10)
            .await;
        assert_eq!(empty.total, 0);
        assert_eq!(service.status().await.indexed_entries, 0);

        fs::remove_dir_all(temp).unwrap();
    }

    #[tokio::test]
    async fn moves_indexed_prefix_without_disk_scan() {
        let (snapshot, temp) =
            snapshot_with_files("search-move-prefix", &[("dir/a.txt", "hello")]).await;
        let service = SearchService::new(true);
        service
            .upsert_virtual_path(snapshot.clone(), "/repo/dir".to_string())
            .await
            .unwrap();
        service
            .upsert_virtual_path(snapshot, "/repo/dir/a.txt".to_string())
            .await
            .unwrap();

        service
            .move_virtual_path("/repo/dir", "/repo/renamed")
            .await
            .unwrap();
        let found = service
            .search(Some("a.txt".to_string()), None, None, 0, 10)
            .await;

        assert_eq!(found.total, 1);
        assert_eq!(found.items[0].path, "/repo/renamed/a.txt");
        assert_eq!(found.items[0].name, "a.txt");

        fs::remove_dir_all(temp).unwrap();
    }

    #[tokio::test]
    async fn cached_normalized_path_supports_case_insensitive_search_after_move() {
        let (snapshot, temp) =
            snapshot_with_files("search-normalized-path", &[("Report.TXT", "hello")]).await;
        let service = SearchService::new(true);
        service
            .upsert_virtual_path(snapshot, "/repo/Report.TXT".to_string())
            .await
            .unwrap();

        let initial = service
            .search(Some("report.txt".to_string()), None, None, 0, 10)
            .await;
        assert_eq!(initial.total, 1);

        service
            .move_virtual_path("/repo/Report.TXT", "/repo/Archive/FINAL.TXT")
            .await
            .unwrap();
        let moved = service
            .search(Some("archive/final.txt".to_string()), None, None, 0, 10)
            .await;
        let old = service
            .search(Some("report.txt".to_string()), None, None, 0, 10)
            .await;

        assert_eq!(moved.total, 1);
        assert_eq!(moved.items[0].path, "/repo/Archive/FINAL.TXT");
        assert_eq!(old.total, 0);

        fs::remove_dir_all(temp).unwrap();
    }

    #[tokio::test]
    async fn removes_mount_entries_without_disk_scan() {
        let (snapshot, temp) =
            snapshot_with_files("search-remove-mount", &[("a.txt", "hello")]).await;
        let service = SearchService::new(true);
        service
            .upsert_virtual_path(snapshot, "/repo/a.txt".to_string())
            .await
            .unwrap();

        service.remove_mount("/repo").await.unwrap();

        let found = service.search(None, None, None, 0, 10).await;
        assert_eq!(found.total, 0);
        assert_eq!(service.status().await.indexed_entries, 0);

        fs::remove_dir_all(temp).unwrap();
    }

    #[tokio::test]
    async fn search_paginates_without_losing_total() {
        let (snapshot, temp) = snapshot_with_files(
            "search-pagination",
            &[("a.txt", "a"), ("b.txt", "b"), ("c.log", "c")],
        )
        .await;
        let service = SearchService::new(true);
        for path in ["/repo/a.txt", "/repo/b.txt", "/repo/c.log"] {
            service
                .upsert_virtual_path(snapshot.clone(), path.to_string())
                .await
                .unwrap();
        }

        let result = service
            .search(Some("txt".to_string()), None, None, 1, 1)
            .await;

        assert_eq!(result.total, 2);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.offset, 1);
        assert_eq!(result.limit, 1);

        fs::remove_dir_all(temp).unwrap();
    }

    #[tokio::test]
    async fn recent_respects_limit() {
        let (snapshot, temp) =
            snapshot_with_files("search-recent-limit", &[("a.txt", "a"), ("b.txt", "b")]).await;
        let service = SearchService::new(true);
        for path in ["/repo/a.txt", "/repo/b.txt"] {
            service
                .upsert_virtual_path(snapshot.clone(), path.to_string())
                .await
                .unwrap();
        }

        let result = service.recent(1).await;

        assert_eq!(result.len(), 1);

        fs::remove_dir_all(temp).unwrap();
    }

    #[tokio::test]
    async fn recent_uses_bounded_window_and_keeps_order() {
        let service = SearchService::new(true);
        {
            let mut entries = service.entries.write().await;
            for (path, modified) in [
                ("/repo/a.txt", "999"),
                ("/repo/b.txt", "1000"),
                ("/repo/c.txt", "200"),
                ("/repo/d.txt", "1000"),
            ] {
                entries.push(IndexedEntry::new(test_search_result(path, modified)));
            }
        }

        let result = service.recent(3).await;
        let paths = result.into_iter().map(|item| item.path).collect::<Vec<_>>();

        assert_eq!(paths, vec!["/repo/b.txt", "/repo/d.txt", "/repo/a.txt"]);
    }

    #[tokio::test]
    async fn cancel_rebuild_marks_status_cancelled() {
        let service = SearchService::new(true);
        {
            let mut status = service.status.write().await;
            status.state = "scanning".to_string();
        }

        service.cancel_rebuild().await.unwrap();

        let status = service.status().await;
        assert_eq!(status.state, "cancelled");
        assert_eq!(status.last_error.as_deref(), Some("索引重建已取消"));
    }

    #[test]
    fn scan_checks_rebuild_token() {
        let token = AtomicU64::new(2);

        let result = ensure_scan_not_cancelled(&token, 1);

        assert!(matches!(result, Err(ScanError::Cancelled)));
    }

    async fn snapshot_with_files(
        prefix: &str,
        files: &[(&str, &str)],
    ) -> (Arc<MappingSnapshot>, PathBuf) {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!("web-file-browser-{prefix}-{nonce}"));
        fs::create_dir_all(&temp).unwrap();
        for (path, content) in files {
            let target = temp.join(path);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(target, content).unwrap();
        }
        let snapshot = MappingSnapshot::build(vec![PathMapping {
            id: Some(1),
            mount_path: "/repo".to_string(),
            folder_path: temp.to_string_lossy().to_string(),
            remark: None,
            order: Some(0),
            writable: true,
        }])
        .await
        .unwrap();
        (snapshot, temp)
    }

    fn test_search_result(path: &str, modified: &str) -> SearchResult {
        SearchResult {
            name: path.rsplit('/').next().unwrap_or_default().to_string(),
            path: path.to_string(),
            extension: Path::new(path)
                .extension()
                .map(|extension| extension.to_string_lossy().to_string())
                .unwrap_or_default(),
            modified: modified.to_string(),
            size: 0,
            entry_type: EntryKind::File.as_str().to_string(),
            mount_path: "/repo".to_string(),
        }
    }
}
