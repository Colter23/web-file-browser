use serde::{Deserialize, Serialize};

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
        folder_total: usize,
        file_total: usize,
        offset: usize,
        limit: usize,
    ) -> Self {
        let has_more = offset.saturating_add(limit) < folder_total
            || offset.saturating_add(limit) < file_total;
        Self {
            path,
            folder,
            file,
            folder_total: Some(folder_total),
            file_total: Some(file_total),
            offset: Some(offset),
            limit: Some(limit),
            has_more: Some(has_more),
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
    pub trash_dir: String,
    pub static_dir: String,
    pub max_upload_bytes: Option<u64>,
    pub max_dir_page_size: usize,
    pub max_dir_concurrency: usize,
    pub max_transfer_concurrency: usize,
    pub max_ip_concurrency: usize,
    pub max_task_concurrency: usize,
    pub task_speed_limit_bytes_per_sec: Option<u64>,
    pub index_enabled: bool,
    pub index_scan_delay_ms: u64,
    pub audit_file: String,
    pub trash_retention_days: Option<u64>,
    pub trash_max_bytes: Option<u64>,
    pub auth_configured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEntryRequest {
    #[serde(rename = "type")]
    pub entry_type: CreateEntryType,
    pub name: String,
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
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteTaskRequest {
    pub paths: Vec<String>,
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
pub struct MetricsResponse {
    pub mappings: usize,
    pub active_sessions: usize,
    pub tasks_total: usize,
    pub tasks_running: usize,
    pub trash_entries: usize,
    pub indexed_entries: usize,
}
