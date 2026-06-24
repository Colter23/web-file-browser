use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    File,
    Folder,
}

impl EntryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Folder => "folder",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathMapping {
    #[serde(default)]
    pub id: Option<i64>,
    pub mount_path: String,
    pub folder_path: String,
    #[serde(default)]
    pub remark: Option<String>,
    #[serde(default)]
    pub order: Option<i32>,
    #[serde(default = "default_writable")]
    pub writable: bool,
}

fn default_writable() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReorderMappingItem {
    pub id: i64,
    pub order: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReorderMappingsRequest {
    pub items: Vec<ReorderMappingItem>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum ConflictPolicy {
    #[default]
    AutoRename,
    Reject,
    Overwrite,
}

impl FromStr for ConflictPolicy {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "autoRename" | "auto-rename" | "auto_rename" | "auto" => Ok(Self::AutoRename),
            "reject" => Ok(Self::Reject),
            "overwrite" => Ok(Self::Overwrite),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FolderData {
    pub path: String,
    pub folder: Vec<FolderInfo>,
    pub file: Vec<FileInfo>,
    #[serde(rename = "folderTotal", skip_serializing_if = "Option::is_none")]
    pub folder_total: Option<usize>,
    #[serde(rename = "fileTotal", skip_serializing_if = "Option::is_none")]
    pub file_total: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    #[serde(rename = "hasMore", skip_serializing_if = "Option::is_none")]
    pub has_more: Option<bool>,
}

#[derive(Debug, Clone, Copy)]
pub struct FolderPageInfo {
    pub folder_total: Option<usize>,
    pub file_total: Option<usize>,
    pub offset: usize,
    pub limit: usize,
    pub has_more: bool,
}

impl FolderData {
    pub fn full(path: String, folder: Vec<FolderInfo>, file: Vec<FileInfo>) -> Self {
        Self {
            path,
            folder,
            file,
            folder_total: None,
            file_total: None,
            offset: None,
            limit: None,
            has_more: None,
        }
    }

    pub fn paged(
        path: String,
        folder: Vec<FolderInfo>,
        file: Vec<FileInfo>,
        page: FolderPageInfo,
    ) -> Self {
        Self {
            path,
            folder,
            file,
            folder_total: page.folder_total,
            file_total: page.file_total,
            offset: Some(page.offset),
            limit: Some(page.limit),
            has_more: Some(page.has_more),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FolderInfo {
    pub name: String,
    pub path: String,
    pub modified: String,
    #[serde(rename = "type")]
    pub entry_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub modified: String,
    pub size: u64,
    pub extension: String,
    #[serde(rename = "type")]
    pub entry_type: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum FolderNode {
    #[serde(rename = "virtual")]
    Virtual {
        name: String,
        path: String,
        children: Vec<FolderNode>,
    },
    #[serde(rename = "real")]
    Real {
        name: String,
        path: String,
        #[serde(rename = "realPath")]
        real_path: String,
    },
}

impl FolderNode {
    pub fn name(&self) -> &str {
        match self {
            Self::Virtual { name, .. } | Self::Real { name, .. } => name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupPasswordRequest {
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponse {
    pub authenticated: bool,
    pub auth_configured: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSettings {
    pub bind_address: String,
    pub port: u16,
    pub mapping_file: String,
    pub config_file: String,
    pub auth_file: String,
    pub favorites_file: String,
    pub trash_dir: String,
    pub static_dir: String,
    pub cors_allowed_origins: Vec<String>,
    pub trust_proxy_headers: bool,
    pub max_edit_bytes: u64,
    pub editable_extensions: Vec<String>,
    pub editable_mime_types: Vec<String>,
    pub max_upload_bytes: Option<u64>,
    pub max_dir_page_size: usize,
    pub max_dir_concurrency: usize,
    pub max_transfer_concurrency: usize,
    pub max_ip_concurrency: usize,
    pub max_task_concurrency: usize,
    pub task_history_limit: usize,
    pub task_speed_limit_bytes_per_sec: Option<u64>,
    pub max_extract_bytes: Option<u64>,
    pub max_extract_files: Option<usize>,
    pub max_extract_depth: usize,
    pub index_enabled: bool,
    pub index_rebuild_on_startup: bool,
    pub index_scan_delay_ms: u64,
    pub audit_file: String,
    pub audit_max_bytes: Option<u64>,
    pub audit_retention_files: usize,
    pub trash_retention_days: Option<u64>,
    pub trash_max_bytes: Option<u64>,
    pub conflict_policy: ConflictPolicy,
    pub auth_configured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEntryRequest {
    #[serde(rename = "type")]
    pub entry_type: CreateEntryType,
    pub name: String,
    #[serde(default, alias = "conflict")]
    pub conflict_policy: Option<ConflictPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CreateEntryType {
    File,
    Folder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveEntryRequest {
    pub target_path: String,
    #[serde(default, alias = "conflict")]
    pub conflict_policy: Option<ConflictPolicy>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileOperationResponse {
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadResponse {
    pub files: Vec<FileOperationResponse>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskPathRequest {
    pub sources: Vec<String>,
    pub target_path: Option<String>,
    #[serde(default, alias = "conflict")]
    pub conflict_policy: Option<ConflictPolicy>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ArchiveFormat {
    TarGz,
    Zip,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveTaskRequest {
    pub sources: Vec<String>,
    pub target_path: String,
    #[serde(default)]
    pub output_name: Option<String>,
    pub format: ArchiveFormat,
    #[serde(default, alias = "conflict")]
    pub conflict_policy: Option<ConflictPolicy>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractTaskRequest {
    pub source_path: String,
    pub target_path: String,
    #[serde(default)]
    pub folder_name: Option<String>,
    #[serde(default, alias = "conflict")]
    pub conflict_policy: Option<ConflictPolicy>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteTaskRequest {
    pub paths: Vec<String>,
    #[serde(default)]
    pub permanent: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskResponse {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TaskKind {
    Copy,
    Move,
    Delete,
    Archive,
    Extract,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TaskState {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskError {
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStatus {
    pub id: String,
    pub kind: TaskKind,
    pub state: TaskState,
    pub progress: f64,
    pub processed_bytes: u64,
    pub total_bytes: u64,
    pub speed_bytes_per_sec: f64,
    pub processed_items: usize,
    pub total_items: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_path: Option<String>,
    pub errors: Vec<TaskError>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub created_at: String,
    pub cancelled: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub name: String,
    pub path: String,
    pub extension: String,
    pub modified: String,
    pub size: u64,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub mount_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub items: Vec<SearchResult>,
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexStatus {
    pub enabled: bool,
    pub state: String,
    pub indexed_entries: usize,
    pub last_started_at: Option<String>,
    pub last_finished_at: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadinessResponse {
    pub status: String,
    pub version: String,
    pub checks: Vec<ReadinessCheck>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadinessCheck {
    pub name: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricsResponse {
    pub mappings: usize,
    pub active_sessions: usize,
    pub trash_entries: usize,
    pub tasks: TaskMetrics,
    pub limits: RequestLimitMetrics,
    pub index: IndexStatus,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskMetrics {
    pub total: usize,
    pub queued: usize,
    pub running: usize,
    pub completed: usize,
    pub failed: usize,
    pub cancelled: usize,
    pub errors_total: usize,
    pub processed_bytes: u64,
    pub current_speed_bytes_per_sec: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestLimitMetrics {
    pub dir_scan_limit: usize,
    pub active_dir_scans: usize,
    pub transfer_limit: usize,
    pub active_transfers: usize,
    pub ip_limit: usize,
    pub tracked_ips: usize,
    pub active_ip_requests: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteItem {
    pub id: String,
    #[serde(default)]
    pub mount_id: Option<i64>,
    pub mount_path: String,
    #[serde(default)]
    pub relative_path: String,
    pub name: String,
    pub order: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteResponse {
    pub id: String,
    pub mount_id: Option<i64>,
    pub mount_path: String,
    pub relative_path: String,
    pub path: String,
    pub name: String,
    pub order: i32,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFavoriteRequest {
    pub path: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub order: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFavoriteRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub order: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReorderFavoriteItem {
    pub id: String,
    pub order: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReorderFavoritesRequest {
    pub items: Vec<ReorderFavoriteItem>,
}
