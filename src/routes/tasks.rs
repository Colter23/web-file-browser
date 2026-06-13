use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde::Serialize;
use std::{
    collections::HashSet,
    fs,
    io::{Read, Write},
    path::{Component, Path as FsPath, PathBuf},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};
use tar::{Archive as TarArchive, Builder as TarBuilder, EntryType, Header};
use tokio::runtime::Handle;
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::{
    app::AppState,
    error::AppError,
    models::{
        ArchiveFormat, ArchiveTaskRequest, ConflictPolicy, DeleteTaskRequest, ExtractTaskRequest,
        MoveEntryRequest, TaskKind, TaskPathRequest, TaskResponse, TaskStatus,
    },
    services::{
        conflict, file_ops,
        path_resolver::{
            self, MappingSnapshot, ensure_file, ensure_folder, ensure_writable, join_virtual_path,
            normalize_child_name,
        },
    },
};

const COPY_BUFFER_SIZE: usize = 256 * 1024;
const MAX_ARCHIVE_DEPTH: usize = 64;
const PROGRESS_REPORT_BYTES: u64 = 1024 * 1024;
const PROGRESS_REPORT_INTERVAL: Duration = Duration::from_millis(200);
const PROGRESS_CANCEL_CHECK_INTERVAL: Duration = Duration::from_millis(50);
const THROTTLE_CANCEL_CHECK_INTERVAL: Duration = Duration::from_millis(200);

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TaskCleanupResponse {
    removed: usize,
}

pub fn task_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/tasks", get(list_tasks))
        .route("/tasks/cleanup", post(cleanup_tasks))
        .route("/tasks/copy", post(create_copy_task))
        .route("/tasks/move", post(create_move_task))
        .route("/tasks/delete", post(create_delete_task))
        .route("/tasks/archive", post(create_archive_task))
        .route("/tasks/extract", post(create_extract_task))
        .route("/tasks/{id}", get(get_task))
        .route("/tasks/{id}/cancel", post(cancel_task))
}

async fn list_tasks(State(state): State<Arc<AppState>>) -> Json<Vec<TaskStatus>> {
    Json(state.tasks.list().await)
}

async fn cleanup_tasks(
    State(state): State<Arc<AppState>>,
) -> Result<Json<TaskCleanupResponse>, AppError> {
    let removed = state.tasks.cleanup_finished().await;
    let detail = format!("removed={removed}");
    state
        .audit
        .record("admin", "task.cleanup", None, Some(&detail))
        .await?;
    Ok(Json(TaskCleanupResponse { removed }))
}

async fn get_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<TaskStatus>, AppError> {
    Ok(Json(state.tasks.get(&id).await?))
}

async fn cancel_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<TaskStatus>, AppError> {
    Ok(Json(state.tasks.cancel(&id).await?))
}

async fn create_copy_task(
    State(state): State<Arc<AppState>>,
    Json(request): Json<TaskPathRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    let target_path = request
        .target_path
        .clone()
        .ok_or_else(|| AppError::bad_request("复制任务需要 targetPath"))?;
    if request.sources.is_empty() {
        return Err(AppError::bad_request("复制任务 sources 不能为空"));
    }
    ensure_non_blank_paths("复制任务 sources", &request.sources)?;
    ensure_non_blank_path("复制任务 targetPath", &target_path)?;
    let task = state
        .tasks
        .create(TaskKind::Copy, request.sources.len(), 0)
        .await?;
    let id = task.id.clone();
    let policy = request
        .conflict_policy
        .unwrap_or(state.runtime_settings.conflict_policy);
    spawn_copy_task(state, id.clone(), request.sources, target_path, policy);
    Ok(Json(TaskResponse { id }))
}

async fn create_move_task(
    State(state): State<Arc<AppState>>,
    Json(request): Json<TaskPathRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    let target_path = request
        .target_path
        .clone()
        .ok_or_else(|| AppError::bad_request("移动任务需要 targetPath"))?;
    if request.sources.is_empty() {
        return Err(AppError::bad_request("移动任务 sources 不能为空"));
    }
    ensure_non_blank_paths("移动任务 sources", &request.sources)?;
    ensure_non_blank_path("移动任务 targetPath", &target_path)?;
    let task = state
        .tasks
        .create(TaskKind::Move, request.sources.len(), 0)
        .await?;
    let id = task.id.clone();
    let policy = request
        .conflict_policy
        .unwrap_or(state.runtime_settings.conflict_policy);
    spawn_move_task(state, id.clone(), request.sources, target_path, policy);
    Ok(Json(TaskResponse { id }))
}

async fn create_delete_task(
    State(state): State<Arc<AppState>>,
    Json(request): Json<DeleteTaskRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    if request.paths.is_empty() {
        return Err(AppError::bad_request("删除任务 paths 不能为空"));
    }
    ensure_non_blank_paths("删除任务 paths", &request.paths)?;
    let task = state
        .tasks
        .create(TaskKind::Delete, request.paths.len(), 0)
        .await?;
    let id = task.id.clone();
    spawn_delete_task(state, id.clone(), request.paths);
    Ok(Json(TaskResponse { id }))
}

async fn create_archive_task(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ArchiveTaskRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    if request.sources.is_empty() {
        return Err(AppError::bad_request("压缩任务 sources 不能为空"));
    }
    ensure_non_blank_paths("压缩任务 sources", &request.sources)?;
    ensure_non_blank_path("压缩任务 targetPath", &request.target_path)?;
    ensure_optional_child_name("压缩任务 outputName", request.output_name.as_deref())?;
    let task = state
        .tasks
        .create(TaskKind::Archive, request.sources.len(), 0)
        .await?;
    let id = task.id.clone();
    let policy = request
        .conflict_policy
        .unwrap_or(state.runtime_settings.conflict_policy);
    spawn_archive_task(state, id.clone(), request, policy);
    Ok(Json(TaskResponse { id }))
}

async fn create_extract_task(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ExtractTaskRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    ensure_non_blank_path("解压任务 sourcePath", &request.source_path)?;
    ensure_non_blank_path("解压任务 targetPath", &request.target_path)?;
    ensure_optional_child_name("解压任务 folderName", request.folder_name.as_deref())?;
    let task = state.tasks.create(TaskKind::Extract, 1, 0).await?;
    let id = task.id.clone();
    let policy = request
        .conflict_policy
        .unwrap_or(state.runtime_settings.conflict_policy);
    spawn_extract_task(state, id.clone(), request, policy);
    Ok(Json(TaskResponse { id }))
}

fn ensure_non_blank_path(label: &str, path: &str) -> Result<(), AppError> {
    if path.trim().is_empty() {
        return Err(AppError::bad_request(format!("{label} 不能为空")));
    }
    Ok(())
}

fn ensure_non_blank_paths(label: &str, paths: &[String]) -> Result<(), AppError> {
    if paths.iter().any(|path| path.trim().is_empty()) {
        return Err(AppError::bad_request(format!("{label} 包含空路径")));
    }
    Ok(())
}

fn ensure_optional_child_name(label: &str, name: Option<&str>) -> Result<(), AppError> {
    let Some(name) = name else {
        return Ok(());
    };
    if name.trim().is_empty() {
        return Err(AppError::bad_request(format!("{label} 不能为空")));
    }
    normalize_child_name(name)
        .map(|_| ())
        .map_err(|error| AppError::bad_request(format!("{label} 无效: {error}")))
}

fn spawn_copy_task(
    state: Arc<AppState>,
    task_id: String,
    sources: Vec<String>,
    target_path: String,
    policy: ConflictPolicy,
) {
    tokio::spawn(async move {
        let Ok(_permit) = state.tasks.acquire_run_permit().await else {
            let _ = state
                .tasks
                .add_error(&task_id, String::new(), "任务调度器不可用".to_string())
                .await;
            let _ = state.tasks.finish(&task_id).await;
            return;
        };
        let _ = state.tasks.mark_running(&task_id).await;
        for source_path in sources {
            if state.tasks.is_cancelled(&task_id).await {
                break;
            }
            let _ = state
                .tasks
                .set_current_path(&task_id, Some(source_path.clone()))
                .await;
            let result = copy_one(
                state.clone(),
                task_id.clone(),
                source_path.clone(),
                target_path.clone(),
                policy,
            )
            .await;
            match result {
                Ok(_) => {
                    let _ = state.tasks.item_done(&task_id, 0).await;
                }
                Err(error) => {
                    let _ = state
                        .tasks
                        .add_error(&task_id, source_path, error.to_string())
                        .await;
                }
            }
        }
        let _ = state.tasks.finish(&task_id).await;
    });
}

fn spawn_move_task(
    state: Arc<AppState>,
    task_id: String,
    sources: Vec<String>,
    target_path: String,
    policy: ConflictPolicy,
) {
    tokio::spawn(async move {
        let Ok(_permit) = state.tasks.acquire_run_permit().await else {
            let _ = state
                .tasks
                .add_error(&task_id, String::new(), "任务调度器不可用".to_string())
                .await;
            let _ = state.tasks.finish(&task_id).await;
            return;
        };
        let _ = state.tasks.mark_running(&task_id).await;
        for source_path in sources {
            if state.tasks.is_cancelled(&task_id).await {
                break;
            }
            let _ = state
                .tasks
                .set_current_path(&task_id, Some(source_path.clone()))
                .await;
            let result = move_one(
                state.clone(),
                task_id.clone(),
                source_path.clone(),
                target_path.clone(),
                policy,
            )
            .await;
            match result {
                Ok(()) => {
                    let _ = state.tasks.item_done(&task_id, 0).await;
                }
                Err(error) => {
                    let _ = state
                        .tasks
                        .add_error(&task_id, source_path, error.to_string())
                        .await;
                }
            }
        }
        let _ = state.tasks.finish(&task_id).await;
    });
}

fn spawn_delete_task(state: Arc<AppState>, task_id: String, paths: Vec<String>) {
    tokio::spawn(async move {
        let Ok(_permit) = state.tasks.acquire_run_permit().await else {
            let _ = state
                .tasks
                .add_error(&task_id, String::new(), "任务调度器不可用".to_string())
                .await;
            let _ = state.tasks.finish(&task_id).await;
            return;
        };
        let _ = state.tasks.mark_running(&task_id).await;
        for path in paths {
            if state.tasks.is_cancelled(&task_id).await {
                break;
            }
            let _ = state
                .tasks
                .set_current_path(&task_id, Some(path.clone()))
                .await;
            let result = delete_one(state.clone(), task_id.clone(), path.clone()).await;
            match result {
                Ok(bytes) => {
                    let _ = state.tasks.item_done(&task_id, bytes).await;
                }
                Err(error) => {
                    let _ = state
                        .tasks
                        .add_error(&task_id, path, error.to_string())
                        .await;
                }
            }
        }
        let _ = state.tasks.finish(&task_id).await;
    });
}

fn spawn_archive_task(
    state: Arc<AppState>,
    task_id: String,
    request: ArchiveTaskRequest,
    policy: ConflictPolicy,
) {
    tokio::spawn(async move {
        let Ok(_permit) = state.tasks.acquire_run_permit().await else {
            let _ = state
                .tasks
                .add_error(&task_id, String::new(), "任务调度器不可用".to_string())
                .await;
            let _ = state.tasks.finish(&task_id).await;
            return;
        };
        let _ = state.tasks.mark_running(&task_id).await;
        let source_summary = request.sources.join(", ");
        let result = archive_task(state.clone(), task_id.clone(), request, policy).await;
        match result {
            Ok(_) => {}
            Err(error) => {
                let _ = state
                    .tasks
                    .add_error(&task_id, source_summary, error.to_string())
                    .await;
            }
        }
        let _ = state.tasks.finish(&task_id).await;
    });
}

fn spawn_extract_task(
    state: Arc<AppState>,
    task_id: String,
    request: ExtractTaskRequest,
    policy: ConflictPolicy,
) {
    tokio::spawn(async move {
        let Ok(_permit) = state.tasks.acquire_run_permit().await else {
            let _ = state
                .tasks
                .add_error(&task_id, String::new(), "任务调度器不可用".to_string())
                .await;
            let _ = state.tasks.finish(&task_id).await;
            return;
        };
        let _ = state.tasks.mark_running(&task_id).await;
        let source_path = request.source_path.clone();
        let result = extract_task(state.clone(), task_id.clone(), request, policy).await;
        match result {
            Ok(_) => {}
            Err(error) => {
                let _ = state
                    .tasks
                    .add_error(&task_id, source_path, error.to_string())
                    .await;
            }
        }
        let _ = state.tasks.finish(&task_id).await;
    });
}

#[derive(Debug, Clone)]
struct ArchiveSource {
    real_path: PathBuf,
    archive_name: String,
}

#[derive(Debug, Clone, Copy)]
struct ExtractLimits {
    max_bytes: Option<u64>,
    max_files: Option<usize>,
    max_depth: usize,
}

async fn archive_task(
    state: Arc<AppState>,
    task_id: String,
    request: ArchiveTaskRequest,
    policy: ConflictPolicy,
) -> Result<u64, AppError> {
    let snapshot = state.mapping_store.snapshot().await;
    let target_parent =
        path_resolver::resolve_existing(snapshot.clone(), request.target_path).await?;
    ensure_writable(&target_parent.mapping)?;
    ensure_folder(&target_parent.real_path, &target_parent.virtual_path)?;

    let mut sources = Vec::with_capacity(request.sources.len());
    let mut archive_names = HashSet::new();
    for source_path in request.sources {
        state
            .tasks
            .set_current_path(&task_id, Some(source_path.clone()))
            .await?;
        let source =
            path_resolver::resolve_existing_no_follow_final(snapshot.clone(), source_path).await?;
        let archive_name = source
            .real_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| AppError::bad_request("压缩源路径没有文件名"))
            .and_then(normalize_child_name)?;
        if !archive_names.insert(archive_name.clone()) {
            return Err(AppError::conflict(format!(
                "压缩源存在重复名称: {archive_name}"
            )));
        }
        sources.push(ArchiveSource {
            real_path: source.real_path,
            archive_name,
        });
    }

    let output_name = archive_output_name(request.output_name, request.format, &sources)?;
    let output_virtual_path = join_virtual_path(&target_parent.virtual_path, &output_name);
    let target = conflict::resolve_child(
        &target_parent.real_path,
        &output_name,
        &output_virtual_path,
        policy,
    )?;
    if target.existed {
        conflict::ensure_file_overwrite_allowed(&target)?;
    }
    let actual_output_virtual_path = join_virtual_path(&target_parent.virtual_path, &target.name);
    for source in &sources {
        if target.path == source.real_path
            || (source.real_path.is_dir() && target.path.starts_with(&source.real_path))
        {
            return Err(AppError::bad_request("不能把压缩包写入待压缩路径内部"));
        }
    }

    let temp_path = temp_sibling_path(&target.path, "archive");
    let target_path = target.path.clone();
    let format = request.format;
    let tasks = state.tasks.clone();
    let speed_limit = state.tasks.speed_limit_bytes_per_sec();
    let handle = Handle::current();
    let bytes = tokio::task::spawn_blocking(move || {
        let mut progress = BlockingProgress::new(handle, tasks, task_id, speed_limit);
        let result = archive_sources_sync(&sources, &temp_path, format, &mut progress);
        match result {
            Ok(bytes) => {
                progress.flush()?;
                finalize_temp_path(&temp_path, &target_path, target.existed)?;
                Ok(bytes)
            }
            Err(error) => {
                let _ = progress.report_pending();
                cleanup_path(&temp_path);
                Err(error)
            }
        }
    })
    .await??;
    state
        .audit
        .record(
            "admin",
            "task.archive",
            Some(&target_parent.virtual_path),
            None,
        )
        .await?;
    index_upsert_ignore(&state, snapshot, &actual_output_virtual_path).await;
    Ok(bytes)
}

async fn extract_task(
    state: Arc<AppState>,
    task_id: String,
    request: ExtractTaskRequest,
    policy: ConflictPolicy,
) -> Result<u64, AppError> {
    let snapshot = state.mapping_store.snapshot().await;
    let source = path_resolver::resolve_existing(snapshot.clone(), request.source_path).await?;
    state
        .tasks
        .set_current_path(&task_id, Some(source.virtual_path.clone()))
        .await?;
    ensure_file(&source.real_path, &source.virtual_path)?;
    let target_parent =
        path_resolver::resolve_existing(snapshot.clone(), request.target_path).await?;
    ensure_writable(&target_parent.mapping)?;
    ensure_folder(&target_parent.real_path, &target_parent.virtual_path)?;

    let folder_name = extract_output_folder_name(request.folder_name, &source.real_path)?;
    let output_virtual_path = join_virtual_path(&target_parent.virtual_path, &folder_name);
    let target = conflict::resolve_child(
        &target_parent.real_path,
        &folder_name,
        &output_virtual_path,
        policy,
    )?;
    if target.existed {
        return Err(AppError::conflict("不支持覆盖解压目录"));
    }
    let actual_output_virtual_path = join_virtual_path(&target_parent.virtual_path, &target.name);

    let temp_path = temp_sibling_path(&target.path, "extract");
    let target_path = target.path.clone();
    let source_path = source.real_path.clone();
    let limits = ExtractLimits {
        max_bytes: state.runtime_settings.max_extract_bytes,
        max_files: state.runtime_settings.max_extract_files,
        max_depth: state.runtime_settings.max_extract_depth,
    };
    let tasks = state.tasks.clone();
    let speed_limit = state.tasks.speed_limit_bytes_per_sec();
    let handle = Handle::current();
    let bytes = tokio::task::spawn_blocking(move || {
        let mut progress = BlockingProgress::new(handle, tasks, task_id, speed_limit);
        let result = extract_archive_sync(&source_path, &temp_path, limits, &mut progress);
        match result {
            Ok(bytes) => {
                progress.flush()?;
                finalize_temp_path(&temp_path, &target_path, false)?;
                Ok(bytes)
            }
            Err(error) => {
                let _ = progress.report_pending();
                cleanup_path(&temp_path);
                Err(error)
            }
        }
    })
    .await??;
    state
        .audit
        .record("admin", "task.extract", Some(&source.virtual_path), None)
        .await?;
    index_upsert_ignore(&state, snapshot, &actual_output_virtual_path).await;
    Ok(bytes)
}

struct BlockingProgress {
    handle: Handle,
    tasks: crate::services::tasks::TaskService,
    task_id: String,
    speed_limit_bytes_per_sec: Option<u64>,
    started: Instant,
    processed_bytes: u64,
    pending_report_bytes: u64,
    last_reported_at: Instant,
    last_cancel_checked_at: Instant,
}

impl BlockingProgress {
    fn new(
        handle: Handle,
        tasks: crate::services::tasks::TaskService,
        task_id: String,
        speed_limit_bytes_per_sec: Option<u64>,
    ) -> Self {
        Self {
            handle,
            tasks,
            task_id,
            speed_limit_bytes_per_sec,
            started: Instant::now(),
            processed_bytes: 0,
            pending_report_bytes: 0,
            last_reported_at: Instant::now(),
            last_cancel_checked_at: Instant::now(),
        }
    }

    fn ensure_not_cancelled(&mut self) -> Result<(), AppError> {
        self.last_cancel_checked_at = Instant::now();
        if self.handle.block_on(self.tasks.is_cancelled(&self.task_id)) {
            Err(AppError::conflict("任务已取消"))
        } else {
            Ok(())
        }
    }

    fn check_cancel_if_due(&mut self) -> Result<(), AppError> {
        if self.last_cancel_checked_at.elapsed() >= PROGRESS_CANCEL_CHECK_INTERVAL {
            self.ensure_not_cancelled()?;
        }
        Ok(())
    }

    fn chunk_done(&mut self, bytes: u64) -> Result<(), AppError> {
        self.check_cancel_if_due()?;
        self.processed_bytes = self.processed_bytes.saturating_add(bytes);
        self.pending_report_bytes = self.pending_report_bytes.saturating_add(bytes);
        self.report_if_due()?;
        if let Some(limit) = self.speed_limit_bytes_per_sec {
            let expected_elapsed =
                Duration::from_secs_f64(self.processed_bytes as f64 / limit as f64);
            let elapsed = self.started.elapsed();
            if expected_elapsed > elapsed {
                self.report_pending()?;
                sleep_until_or_cancelled(
                    &self.handle,
                    &self.tasks,
                    &self.task_id,
                    self.started + expected_elapsed,
                )?;
                self.last_cancel_checked_at = Instant::now();
            }
        }
        self.check_cancel_if_due()?;
        Ok(())
    }

    fn report_if_due(&mut self) -> Result<(), AppError> {
        if self.pending_report_bytes >= PROGRESS_REPORT_BYTES
            || self.last_reported_at.elapsed() >= PROGRESS_REPORT_INTERVAL
        {
            self.report_pending()?;
        }
        Ok(())
    }

    fn report_pending(&mut self) -> Result<(), AppError> {
        if self.pending_report_bytes == 0 {
            return Ok(());
        }
        let bytes = self.pending_report_bytes;
        self.handle
            .block_on(self.tasks.bytes_done(&self.task_id, bytes))?;
        self.pending_report_bytes = 0;
        self.last_reported_at = Instant::now();
        Ok(())
    }

    fn flush(&mut self) -> Result<(), AppError> {
        self.ensure_not_cancelled()?;
        self.report_pending()
    }

    fn item_done(&mut self) -> Result<(), AppError> {
        self.ensure_not_cancelled()?;
        self.handle.block_on(self.tasks.item_done(&self.task_id, 0))
    }

    fn set_current_path(&mut self, current_path: String) -> Result<(), AppError> {
        self.ensure_not_cancelled()?;
        self.handle.block_on(
            self.tasks
                .set_current_path(&self.task_id, Some(current_path)),
        )
    }
}

fn archive_output_name(
    output_name: Option<String>,
    format: ArchiveFormat,
    sources: &[ArchiveSource],
) -> Result<String, AppError> {
    let raw_name = match output_name {
        Some(name) => name,
        None if sources.len() == 1 => {
            format!("{}{}", sources[0].archive_name, archive_extension(format))
        }
        None => format!("archive{}", archive_extension(format)),
    };
    let name = normalize_child_name(&raw_name)?;
    Ok(ensure_archive_extension(&name, format))
}

fn ensure_archive_extension(name: &str, format: ArchiveFormat) -> String {
    match format {
        ArchiveFormat::TarGz => {
            if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
                name.to_string()
            } else {
                format!("{name}.tar.gz")
            }
        }
        ArchiveFormat::Zip => {
            if name.ends_with(".zip") {
                name.to_string()
            } else {
                format!("{name}.zip")
            }
        }
    }
}

fn archive_extension(format: ArchiveFormat) -> &'static str {
    match format {
        ArchiveFormat::TarGz => ".tar.gz",
        ArchiveFormat::Zip => ".zip",
    }
}

fn extract_output_folder_name(
    folder_name: Option<String>,
    source_path: &FsPath,
) -> Result<String, AppError> {
    let raw_name = match folder_name {
        Some(name) => name,
        None => {
            let file_name = source_path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| AppError::bad_request("压缩包路径没有文件名"))?;
            archive_stem(file_name).to_string()
        }
    };
    normalize_child_name(&raw_name)
}

fn archive_stem(file_name: &str) -> &str {
    file_name
        .strip_suffix(".tar.gz")
        .or_else(|| file_name.strip_suffix(".tgz"))
        .or_else(|| file_name.strip_suffix(".zip"))
        .unwrap_or(file_name)
}

fn archive_sources_sync(
    sources: &[ArchiveSource],
    temp_path: &FsPath,
    format: ArchiveFormat,
    progress: &mut BlockingProgress,
) -> Result<u64, AppError> {
    progress.ensure_not_cancelled()?;
    if let Some(parent) = temp_path.parent() {
        fs::create_dir_all(parent)?;
    }
    match format {
        ArchiveFormat::TarGz => archive_tar_gz_sync(sources, temp_path, progress),
        ArchiveFormat::Zip => archive_zip_sync(sources, temp_path, progress),
    }
}

fn archive_tar_gz_sync(
    sources: &[ArchiveSource],
    temp_path: &FsPath,
    progress: &mut BlockingProgress,
) -> Result<u64, AppError> {
    progress.ensure_not_cancelled()?;
    let file = fs::File::create(temp_path)?;
    let encoder = GzEncoder::new(file, Compression::fast());
    let mut builder = TarBuilder::new(encoder);
    let mut bytes = 0;
    let mut used_paths = HashSet::new();
    for source in sources {
        progress.ensure_not_cancelled()?;
        bytes += append_tar_path(
            &mut builder,
            &source.real_path,
            FsPath::new(&source.archive_name),
            &mut used_paths,
            progress,
        )?;
        progress.item_done()?;
    }
    progress.ensure_not_cancelled()?;
    builder.finish()?;
    progress.ensure_not_cancelled()?;
    builder.into_inner()?.finish()?;
    Ok(bytes)
}

fn append_tar_path<W: Write>(
    builder: &mut TarBuilder<W>,
    source: &FsPath,
    archive_path: &FsPath,
    used_paths: &mut HashSet<PathBuf>,
    progress: &mut BlockingProgress,
) -> Result<u64, AppError> {
    progress.ensure_not_cancelled()?;
    let metadata = fs::symlink_metadata(source)?;
    if metadata.file_type().is_symlink() {
        return Err(AppError::bad_request(format!(
            "不支持压缩符号链接: {}",
            source.display()
        )));
    }
    let archive_path = normalize_archive_path(archive_path)?;
    progress.set_current_path(archive_path_string(&archive_path))?;
    if !used_paths.insert(archive_path.clone()) {
        return Err(AppError::conflict(format!(
            "压缩源存在重复归档路径: {}",
            archive_path.display()
        )));
    }
    if metadata.is_dir() {
        progress.ensure_not_cancelled()?;
        builder.append_dir(&archive_path, source)?;
        let mut bytes = 0;
        for entry in fs::read_dir(source)? {
            progress.ensure_not_cancelled()?;
            let entry = entry?;
            bytes += append_tar_path(
                builder,
                &entry.path(),
                &archive_path.join(entry.file_name()),
                used_paths,
                progress,
            )?;
        }
        return Ok(bytes);
    }
    if !metadata.is_file() {
        return Err(AppError::bad_request(format!(
            "不支持压缩此类文件: {}",
            source.display()
        )));
    }

    let mut header = Header::new_gnu();
    header.set_metadata(&metadata);
    header.set_size(metadata.len());
    header.set_cksum();
    progress.ensure_not_cancelled()?;
    let reader = ProgressReader {
        inner: fs::File::open(source)?,
        progress,
    };
    builder.append_data(&mut header, &archive_path, reader)?;
    progress.ensure_not_cancelled()?;
    Ok(metadata.len())
}

fn archive_zip_sync(
    sources: &[ArchiveSource],
    temp_path: &FsPath,
    progress: &mut BlockingProgress,
) -> Result<u64, AppError> {
    progress.ensure_not_cancelled()?;
    let file = fs::File::create(temp_path)?;
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(1));
    let mut bytes = 0;
    let mut used_paths = HashSet::new();
    for source in sources {
        progress.ensure_not_cancelled()?;
        bytes += append_zip_path(
            &mut writer,
            &source.real_path,
            FsPath::new(&source.archive_name),
            options,
            &mut used_paths,
            progress,
        )?;
        progress.item_done()?;
    }
    progress.ensure_not_cancelled()?;
    writer.finish()?;
    Ok(bytes)
}

fn append_zip_path<W: Write + std::io::Seek>(
    writer: &mut ZipWriter<W>,
    source: &FsPath,
    archive_path: &FsPath,
    options: SimpleFileOptions,
    used_paths: &mut HashSet<PathBuf>,
    progress: &mut BlockingProgress,
) -> Result<u64, AppError> {
    progress.ensure_not_cancelled()?;
    let metadata = fs::symlink_metadata(source)?;
    if metadata.file_type().is_symlink() {
        return Err(AppError::bad_request(format!(
            "不支持压缩符号链接: {}",
            source.display()
        )));
    }
    let archive_path = normalize_archive_path(archive_path)?;
    progress.set_current_path(archive_path_string(&archive_path))?;
    if !used_paths.insert(archive_path.clone()) {
        return Err(AppError::conflict(format!(
            "压缩源存在重复归档路径: {}",
            archive_path.display()
        )));
    }
    let archive_name = archive_path_string(&archive_path);
    if metadata.is_dir() {
        progress.ensure_not_cancelled()?;
        writer.add_directory(ensure_trailing_slash(&archive_name), options)?;
        let mut bytes = 0;
        for entry in fs::read_dir(source)? {
            progress.ensure_not_cancelled()?;
            let entry = entry?;
            bytes += append_zip_path(
                writer,
                &entry.path(),
                &archive_path.join(entry.file_name()),
                options,
                used_paths,
                progress,
            )?;
        }
        return Ok(bytes);
    }
    if !metadata.is_file() {
        return Err(AppError::bad_request(format!(
            "不支持压缩此类文件: {}",
            source.display()
        )));
    }
    progress.ensure_not_cancelled()?;
    writer.start_file(archive_name, options)?;
    copy_reader_to_writer(fs::File::open(source)?, writer, progress)?;
    progress.ensure_not_cancelled()?;
    Ok(metadata.len())
}

fn extract_archive_sync(
    source_path: &FsPath,
    output_root: &FsPath,
    limits: ExtractLimits,
    progress: &mut BlockingProgress,
) -> Result<u64, AppError> {
    progress.ensure_not_cancelled()?;
    fs::create_dir_all(output_root)?;
    let name = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let bytes = if name.ends_with(".zip") {
        extract_zip_sync(source_path, output_root, limits, progress)?
    } else if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        extract_tar_gz_sync(source_path, output_root, limits, progress)?
    } else {
        return Err(AppError::bad_request("仅支持解压 zip、tar.gz 和 tgz"));
    };
    progress.ensure_not_cancelled()?;
    progress.item_done()?;
    Ok(bytes)
}

fn extract_tar_gz_sync(
    source_path: &FsPath,
    output_root: &FsPath,
    limits: ExtractLimits,
    progress: &mut BlockingProgress,
) -> Result<u64, AppError> {
    progress.ensure_not_cancelled()?;
    let file = fs::File::open(source_path)?;
    let decoder = GzDecoder::new(file);
    let mut archive = TarArchive::new(decoder);
    let mut guard = ExtractGuard::new(limits);
    let mut bytes = 0;
    for entry in archive.entries()? {
        progress.ensure_not_cancelled()?;
        let mut entry = entry?;
        let entry_type = entry.header().entry_type();
        if !matches!(entry_type, EntryType::Regular | EntryType::Directory) {
            return Err(AppError::bad_request("压缩包包含不支持的条目类型"));
        }
        if entry.path_bytes().as_ref().contains(&b'\\') {
            return Err(AppError::bad_request("压缩包条目路径包含非法片段"));
        }
        let entry_path = entry.path()?;
        let relative_path =
            normalize_archive_path_with_depth(entry_path.as_ref(), limits.max_depth)?;
        progress.set_current_path(archive_path_string(&relative_path))?;
        let target = output_root.join(&relative_path);
        if entry_type == EntryType::Directory {
            guard.entry_seen(0)?;
            progress.ensure_not_cancelled()?;
            fs::create_dir_all(&target)?;
            continue;
        }
        let entry_size = entry.size();
        guard.entry_seen(entry_size)?;
        write_extract_file(&mut entry, &target, progress)?;
        bytes += entry_size;
    }
    Ok(bytes)
}

fn extract_zip_sync(
    source_path: &FsPath,
    output_root: &FsPath,
    limits: ExtractLimits,
    progress: &mut BlockingProgress,
) -> Result<u64, AppError> {
    progress.ensure_not_cancelled()?;
    let file = fs::File::open(source_path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut guard = ExtractGuard::new(limits);
    let mut bytes = 0;
    for index in 0..archive.len() {
        progress.ensure_not_cancelled()?;
        let mut file = archive.by_index(index)?;
        reject_zip_symlink(&file)?;
        let relative_path = normalize_archive_entry_name_with_depth(file.name(), limits.max_depth)?;
        progress.set_current_path(archive_path_string(&relative_path))?;
        let target = output_root.join(&relative_path);
        if file.is_dir() {
            guard.entry_seen(0)?;
            progress.ensure_not_cancelled()?;
            fs::create_dir_all(&target)?;
            continue;
        }
        let file_size = file.size();
        guard.entry_seen(file_size)?;
        write_extract_file(&mut file, &target, progress)?;
        bytes += file_size;
    }
    Ok(bytes)
}

struct ExtractGuard {
    limits: ExtractLimits,
    files: usize,
    bytes: u64,
}

impl ExtractGuard {
    fn new(limits: ExtractLimits) -> Self {
        Self {
            limits,
            files: 0,
            bytes: 0,
        }
    }

    fn entry_seen(&mut self, bytes: u64) -> Result<(), AppError> {
        self.files = self.files.saturating_add(1);
        self.bytes = self.bytes.saturating_add(bytes);
        if let Some(max_files) = self.limits.max_files
            && self.files > max_files
        {
            return Err(AppError::payload_too_large("解压文件数量超过限制"));
        }
        if let Some(max_bytes) = self.limits.max_bytes
            && self.bytes > max_bytes
        {
            return Err(AppError::payload_too_large("解压总字节数超过限制"));
        }
        Ok(())
    }
}

fn write_extract_file<R: Read>(
    reader: &mut R,
    target: &FsPath,
    progress: &mut BlockingProgress,
) -> Result<(), AppError> {
    progress.ensure_not_cancelled()?;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    if target.exists() {
        return Err(AppError::conflict(format!(
            "压缩包内存在重复路径: {}",
            target.display()
        )));
    }
    progress.ensure_not_cancelled()?;
    let mut writer = fs::File::create(target)?;
    copy_reader_to_writer(reader, &mut writer, progress)?;
    progress.ensure_not_cancelled()?;
    writer.flush()?;
    Ok(())
}

struct ProgressReader<'a, R> {
    inner: R,
    progress: &'a mut BlockingProgress,
}

impl<R: Read> Read for ProgressReader<'_, R> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if let Err(error) = self.progress.check_cancel_if_due() {
            return Err(std::io::Error::other(error.to_string()));
        }
        let read = self.inner.read(buffer)?;
        if read > 0
            && let Err(error) = self.progress.chunk_done(read as u64)
        {
            return Err(std::io::Error::other(error.to_string()));
        }
        Ok(read)
    }
}

fn copy_reader_to_writer<R: Read, W: Write>(
    mut reader: R,
    writer: &mut W,
    progress: &mut BlockingProgress,
) -> Result<u64, AppError> {
    progress.ensure_not_cancelled()?;
    let mut buffer = vec![0_u8; COPY_BUFFER_SIZE];
    let mut bytes = 0;
    loop {
        progress.check_cancel_if_due()?;
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        progress.check_cancel_if_due()?;
        writer.write_all(&buffer[..read])?;
        progress.chunk_done(read as u64)?;
        bytes += read as u64;
    }
    progress.ensure_not_cancelled()?;
    Ok(bytes)
}

fn normalize_archive_path(path: &FsPath) -> Result<PathBuf, AppError> {
    normalize_archive_path_with_depth(path, MAX_ARCHIVE_DEPTH)
}

fn normalize_archive_path_with_depth(path: &FsPath, max_depth: usize) -> Result<PathBuf, AppError> {
    let mut normalized = PathBuf::new();
    let mut depth = 0;
    for component in path.components() {
        match component {
            Component::Normal(part) => {
                let part = part.to_string_lossy();
                if part.is_empty() || part == "." || part == ".." || part.contains('\\') {
                    return Err(AppError::bad_request("压缩包条目路径包含非法片段"));
                }
                normalized.push(part.as_ref());
                depth += 1;
                if depth > max_depth {
                    return Err(AppError::bad_request("压缩包条目路径过深"));
                }
            }
            Component::CurDir => {}
            _ => return Err(AppError::bad_request("压缩包条目路径不能越界")),
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err(AppError::bad_request("压缩包条目路径不能为空"));
    }
    Ok(normalized)
}

fn normalize_archive_entry_name_with_depth(
    name: &str,
    max_depth: usize,
) -> Result<PathBuf, AppError> {
    if name.contains('\\') {
        return Err(AppError::bad_request("压缩包条目路径包含非法片段"));
    }
    normalize_archive_path_with_depth(FsPath::new(name), max_depth)
}

fn archive_path_string(path: &FsPath) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn ensure_trailing_slash(path: &str) -> String {
    if path.ends_with('/') {
        path.to_string()
    } else {
        format!("{path}/")
    }
}

fn reject_zip_symlink(file: &zip::read::ZipFile<'_>) -> Result<(), AppError> {
    if let Some(mode) = file.unix_mode()
        && mode & 0o170000 == 0o120000
    {
        return Err(AppError::bad_request("压缩包包含不支持的符号链接"));
    }
    Ok(())
}

fn sleep_until_or_cancelled(
    handle: &Handle,
    tasks: &crate::services::tasks::TaskService,
    task_id: &str,
    deadline: Instant,
) -> Result<(), AppError> {
    loop {
        if handle.block_on(tasks.is_cancelled(task_id)) {
            return Err(AppError::conflict("任务已取消"));
        }

        let now = Instant::now();
        if now >= deadline {
            return Ok(());
        }

        thread::sleep((deadline - now).min(THROTTLE_CANCEL_CHECK_INTERVAL));
    }
}

fn temp_sibling_path(path: &FsPath, kind: &str) -> PathBuf {
    let parent = path.parent().unwrap_or_else(|| FsPath::new("."));
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("output");
    parent.join(format!(".{name}.{kind}.tmp-{}", uuid::Uuid::new_v4()))
}

fn finalize_temp_path(
    temp_path: &FsPath,
    target_path: &FsPath,
    replace_file: bool,
) -> Result<(), AppError> {
    if replace_file {
        conflict::replace_file_sync(temp_path, target_path)
    } else {
        if target_path.exists() {
            cleanup_path(temp_path);
            return Err(AppError::conflict(format!(
                "目标路径已存在: {}",
                target_path.display()
            )));
        }
        fs::rename(temp_path, target_path)?;
        Ok(())
    }
}

fn cleanup_path(path: &FsPath) {
    if path.is_dir() {
        let _ = fs::remove_dir_all(path);
    } else if path.exists() {
        let _ = fs::remove_file(path);
    }
}

async fn copy_one(
    state: Arc<AppState>,
    task_id: String,
    source_path: String,
    target_path: String,
    policy: ConflictPolicy,
) -> Result<u64, AppError> {
    let snapshot = state.mapping_store.snapshot().await;
    let source =
        path_resolver::resolve_existing_no_follow_final(snapshot.clone(), source_path).await?;
    let target_parent = path_resolver::resolve_existing(snapshot.clone(), target_path).await?;
    ensure_writable(&target_parent.mapping)?;
    ensure_folder(&target_parent.real_path, &target_parent.virtual_path)?;
    let name = source
        .real_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| AppError::bad_request("源路径没有文件名"))
        .and_then(normalize_child_name)?;
    let desired_virtual_path = join_virtual_path(&target_parent.virtual_path, &name);
    let target = conflict::resolve_child(
        &target_parent.real_path,
        &name,
        &desired_virtual_path,
        policy,
    )?;
    if source.real_path.is_dir() && target.path.starts_with(&source.real_path) {
        return Err(AppError::bad_request("不能把文件夹复制到自身内部"));
    }
    if target.existed {
        if source.real_path.is_dir() {
            return Err(AppError::conflict("不支持覆盖复制目录"));
        }
        conflict::ensure_file_overwrite_allowed(&target)?;
    }
    let source_real_path = source.real_path.clone();
    let temp_path = temp_sibling_path(&target.path, "copy");
    let target_real_path = target.path.clone();
    let replace_existing = target.existed;
    let copied_virtual_path = join_virtual_path(&target_parent.virtual_path, &target.name);
    let tasks = state.tasks.clone();
    let speed_limit = state.tasks.speed_limit_bytes_per_sec();
    let handle = Handle::current();
    let bytes = tokio::task::spawn_blocking(move || {
        let mut progress = BlockingProgress::new(handle, tasks, task_id, speed_limit);
        let result = copy_path_streaming(&source_real_path, &temp_path, &mut progress);
        match result {
            Ok(bytes) => {
                progress.flush()?;
                if let Err(error) =
                    finalize_temp_path(&temp_path, &target_real_path, replace_existing)
                {
                    cleanup_path(&temp_path);
                    return Err(error);
                }
                Ok(bytes)
            }
            Err(error) => {
                let _ = progress.report_pending();
                cleanup_path(&temp_path);
                Err(error)
            }
        }
    })
    .await??;
    state
        .audit
        .record("admin", "task.copy", Some(&source.virtual_path), None)
        .await?;
    index_upsert_ignore(&state, snapshot, &copied_virtual_path).await;
    Ok(bytes)
}

async fn move_one(
    state: Arc<AppState>,
    task_id: String,
    source_path: String,
    target_path: String,
    policy: ConflictPolicy,
) -> Result<(), AppError> {
    state.tasks.ensure_not_cancelled(&task_id).await?;
    let snapshot = state.mapping_store.snapshot().await;
    let source = path_resolver::resolve_existing(snapshot.clone(), source_path.clone()).await?;
    let name = source
        .real_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| AppError::bad_request("源路径没有文件名"))
        .and_then(normalize_child_name)?;
    let target = join_virtual_path(&target_path, &name);
    state.tasks.ensure_not_cancelled(&task_id).await?;
    let response = file_ops::move_entry(
        snapshot.clone(),
        source_path.clone(),
        MoveEntryRequest {
            target_path: target,
            conflict_policy: None,
        },
        policy,
    )
    .await?;
    index_move_ignore(&state, &source_path, &response.path).await;
    index_upsert_ignore(&state, snapshot, &response.path).await;
    state
        .audit
        .record("admin", "task.move", Some(&source.virtual_path), None)
        .await?;
    Ok(())
}

async fn delete_one(state: Arc<AppState>, task_id: String, path: String) -> Result<u64, AppError> {
    let snapshot = state.mapping_store.snapshot().await;
    let target = file_ops::resolve_delete_target(snapshot, path).await?;
    state.tasks.ensure_not_cancelled(&task_id).await?;
    let original_virtual_path = target.virtual_path.clone();
    let original_real_path = target.real_path.to_string_lossy().to_string();
    state
        .trash
        .move_to_trash(
            target.real_path,
            original_virtual_path.clone(),
            original_real_path,
            "admin".to_string(),
        )
        .await?;
    state
        .audit
        .record("admin", "task.delete", Some(&original_virtual_path), None)
        .await?;
    index_remove_ignore(&state, &original_virtual_path).await;
    Ok(0)
}

async fn index_upsert_ignore(state: &AppState, snapshot: Arc<MappingSnapshot>, virtual_path: &str) {
    if let Err(error) = state
        .search
        .upsert_virtual_path(snapshot, virtual_path.to_string())
        .await
    {
        tracing::warn!("更新搜索索引失败: {error}");
    }
}

async fn index_remove_ignore(state: &AppState, virtual_path: &str) {
    if let Err(error) = state.search.remove_virtual_path(virtual_path).await {
        tracing::warn!("移除搜索索引失败: {error}");
    }
}

async fn index_move_ignore(state: &AppState, old_path: &str, new_path: &str) {
    if let Err(error) = state.search.move_virtual_path(old_path, new_path).await {
        tracing::warn!("移动搜索索引失败: {error}");
    }
}

fn copy_path_streaming(
    source: &FsPath,
    target: &FsPath,
    progress: &mut BlockingProgress,
) -> Result<u64, AppError> {
    progress.ensure_not_cancelled()?;
    let metadata = fs::symlink_metadata(source)?;
    if metadata.file_type().is_symlink() {
        return Err(AppError::bad_request(format!(
            "不支持复制符号链接: {}",
            source.display()
        )));
    }
    if metadata.is_dir() {
        copy_dir_streaming(source, target, progress)
    } else if metadata.is_file() {
        copy_file_streaming(source, target, progress)
    } else {
        Err(AppError::bad_request(format!(
            "不支持复制此类文件: {}",
            source.display()
        )))
    }
}

fn copy_dir_streaming(
    source: &FsPath,
    target: &FsPath,
    progress: &mut BlockingProgress,
) -> Result<u64, AppError> {
    fs::create_dir_all(target)?;
    progress.ensure_not_cancelled()?;
    let mut bytes = 0;
    for entry in fs::read_dir(source)? {
        progress.ensure_not_cancelled()?;
        let entry = entry?;
        bytes += copy_path_streaming(&entry.path(), &target.join(entry.file_name()), progress)?;
    }
    Ok(bytes)
}

fn copy_file_streaming(
    source: &FsPath,
    target: &FsPath,
    progress: &mut BlockingProgress,
) -> Result<u64, AppError> {
    progress.ensure_not_cancelled()?;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut reader = fs::File::open(source)?;
    let mut writer = fs::File::create(target)?;
    let mut buffer = vec![0_u8; COPY_BUFFER_SIZE];
    let mut bytes = 0;
    loop {
        progress.check_cancel_if_due()?;
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        progress.check_cancel_if_due()?;
        writer.write_all(&buffer[..read])?;
        progress.chunk_done(read as u64)?;
        bytes += read as u64;
    }
    progress.ensure_not_cancelled()?;
    writer.flush()?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::tasks::TaskService;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn archive_path_rejects_traversal() {
        assert!(normalize_archive_path(FsPath::new("../evil.txt")).is_err());
        assert!(normalize_archive_path(FsPath::new("/absolute.txt")).is_err());
        assert!(
            normalize_archive_entry_name_with_depth("folder\\evil.txt", MAX_ARCHIVE_DEPTH).is_err()
        );

        let normalized = normalize_archive_path(FsPath::new("folder/file.txt")).unwrap();
        assert_eq!(normalized, PathBuf::from("folder").join("file.txt"));
    }

    #[test]
    fn archive_path_rejects_configured_depth_overflow() {
        let ok = normalize_archive_path_with_depth(FsPath::new("a/b.txt"), 2).unwrap();
        assert_eq!(ok, PathBuf::from("a").join("b.txt"));

        let error =
            normalize_archive_entry_name_with_depth("a/b/c.txt", 2).expect_err("应该拒绝过深路径");
        assert!(error.to_string().contains("压缩包条目路径过深"));
    }

    #[tokio::test]
    async fn zip_extract_rejects_zip_slip_entry() {
        let temp = temp_dir("web-file-browser-zip-slip-test");
        fs::create_dir_all(&temp).unwrap();
        let zip_path = temp.join("bad.zip");
        create_zip(&zip_path, &[("../evil.txt", b"bad".as_slice())]);

        let output = temp.join("out");
        let result = run_extract_zip(
            &zip_path,
            &output,
            ExtractLimits {
                max_bytes: None,
                max_files: None,
                max_depth: MAX_ARCHIVE_DEPTH,
            },
        )
        .await;

        assert!(result.is_err());
        assert!(!temp.join("evil.txt").exists());
        fs::remove_dir_all(temp).unwrap();
    }

    #[tokio::test]
    async fn zip_archive_writes_streamed_file() {
        let temp = temp_dir("web-file-browser-zip-archive-test");
        let source_dir = temp.join("source");
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("hello.txt"), "hello").unwrap();
        let target = temp.join("archive.zip");
        let source = ArchiveSource {
            real_path: source_dir,
            archive_name: "source".to_string(),
        };

        let bytes = run_archive_zip(&[source], &target).await.unwrap();

        assert_eq!(bytes, 5);
        let archive_file = fs::File::open(&target).unwrap();
        let mut archive = ZipArchive::new(archive_file).unwrap();
        let mut file = archive.by_name("source/hello.txt").unwrap();
        let mut text = String::new();
        file.read_to_string(&mut text).unwrap();
        assert_eq!(text, "hello");
        fs::remove_dir_all(temp).unwrap();
    }

    #[tokio::test]
    async fn cancelled_archive_stops_before_creating_output() {
        let temp = temp_dir("web-file-browser-cancelled-archive-test");
        let source_dir = temp.join("source");
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("hello.txt"), "hello").unwrap();
        let target = temp.join("archive.zip");
        let source = ArchiveSource {
            real_path: source_dir,
            archive_name: "source".to_string(),
        };

        let service = TaskService::new(1, None, 200);
        let task = service.create(TaskKind::Archive, 1, 0).await.unwrap();
        service.mark_running(&task.id).await.unwrap();
        service.cancel(&task.id).await.unwrap();
        let task_id = task.id;
        let handle = Handle::current();
        let result = tokio::task::spawn_blocking({
            let target = target.clone();
            move || {
                let mut progress = BlockingProgress::new(handle, service, task_id, None);
                archive_zip_sync(&[source], &target, &mut progress)
            }
        })
        .await
        .unwrap();

        assert!(result.is_err());
        assert!(!target.exists());
        fs::remove_dir_all(temp).unwrap();
    }

    #[tokio::test]
    async fn cancelled_extract_stops_before_creating_output_root() {
        let temp = temp_dir("web-file-browser-cancelled-extract-test");
        fs::create_dir_all(&temp).unwrap();
        let zip_path = temp.join("source.zip");
        create_zip(&zip_path, &[("hello.txt", b"hello".as_slice())]);
        let output = temp.join("out");

        let service = TaskService::new(1, None, 200);
        let task = service.create(TaskKind::Extract, 1, 0).await.unwrap();
        service.mark_running(&task.id).await.unwrap();
        service.cancel(&task.id).await.unwrap();
        let task_id = task.id;
        let handle = Handle::current();
        let result = tokio::task::spawn_blocking({
            let zip_path = zip_path.clone();
            let output = output.clone();
            move || {
                let mut progress = BlockingProgress::new(handle, service, task_id, None);
                extract_archive_sync(
                    &zip_path,
                    &output,
                    ExtractLimits {
                        max_bytes: None,
                        max_files: None,
                        max_depth: MAX_ARCHIVE_DEPTH,
                    },
                    &mut progress,
                )
            }
        })
        .await
        .unwrap();

        assert!(result.is_err());
        assert!(!output.exists());
        fs::remove_dir_all(temp).unwrap();
    }

    #[tokio::test]
    async fn throttled_wait_checks_cancel_during_sleep() {
        let service = TaskService::new(1, None, 200);
        let task = service.create(TaskKind::Copy, 1, 0).await.unwrap();
        service.mark_running(&task.id).await.unwrap();

        let handle = Handle::current();
        let task_id = task.id.clone();
        let service_for_worker = service.clone();
        let started = Instant::now();
        let worker = tokio::task::spawn_blocking(move || {
            sleep_until_or_cancelled(
                &handle,
                &service_for_worker,
                &task_id,
                started + Duration::from_secs(5),
            )
        });

        tokio::time::sleep(Duration::from_millis(50)).await;
        service.cancel(&task.id).await.unwrap();

        let result = worker.await.unwrap();
        assert!(result.is_err());
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    #[tokio::test]
    async fn blocking_progress_batches_status_updates_and_flushes_exact_bytes() {
        let service = TaskService::new(1, None, 200);
        let task = service.create(TaskKind::Archive, 1, 0).await.unwrap();
        service.mark_running(&task.id).await.unwrap();

        let handle = Handle::current();
        let inspect_handle = handle.clone();
        let inspect_service = service.clone();
        let task_id = task.id.clone();
        let result = tokio::task::spawn_blocking(move || {
            let mut progress = BlockingProgress::new(handle, service, task_id.clone(), None);
            progress.chunk_done(PROGRESS_REPORT_BYTES / 2)?;
            let before_threshold = inspect_handle
                .block_on(inspect_service.get(&task_id))?
                .processed_bytes;

            progress.chunk_done(PROGRESS_REPORT_BYTES / 2)?;
            let at_threshold = inspect_handle
                .block_on(inspect_service.get(&task_id))?
                .processed_bytes;

            progress.chunk_done(123)?;
            let before_flush = inspect_handle
                .block_on(inspect_service.get(&task_id))?
                .processed_bytes;
            progress.flush()?;
            let after_flush = inspect_handle
                .block_on(inspect_service.get(&task_id))?
                .processed_bytes;

            Ok::<_, AppError>((before_threshold, at_threshold, before_flush, after_flush))
        })
        .await
        .unwrap()
        .unwrap();

        assert_eq!(result.0, 0);
        assert_eq!(result.1, PROGRESS_REPORT_BYTES);
        assert_eq!(result.2, PROGRESS_REPORT_BYTES);
        assert_eq!(result.3, PROGRESS_REPORT_BYTES + 123);
    }

    #[tokio::test]
    async fn copy_path_streaming_copies_regular_file() {
        let temp = temp_dir("web-file-browser-copy-file-test");
        fs::create_dir_all(&temp).unwrap();
        let source = temp.join("source.txt");
        let target = temp.join("target.txt");
        fs::write(&source, "hello").unwrap();

        let bytes = run_copy_path(&source, &target).await.unwrap();

        assert_eq!(bytes, 5);
        assert_eq!(fs::read_to_string(&target).unwrap(), "hello");
        fs::remove_dir_all(temp).unwrap();
    }

    #[tokio::test]
    async fn copy_path_streaming_rejects_symlink_when_available() {
        let temp = temp_dir("web-file-browser-copy-symlink-test");
        fs::create_dir_all(&temp).unwrap();
        let source = temp.join("source.txt");
        let link = temp.join("source-link.txt");
        let target = temp.join("target.txt");
        fs::write(&source, "hello").unwrap();
        if !try_create_file_symlink(&source, &link) {
            fs::remove_dir_all(temp).unwrap();
            return;
        }

        let error = run_copy_path(&link, &target).await.unwrap_err();

        assert!(error.to_string().contains("不支持复制符号链接"));
        assert!(!target.exists());
        fs::remove_dir_all(temp).unwrap();
    }

    async fn run_archive_zip(sources: &[ArchiveSource], target: &FsPath) -> Result<u64, AppError> {
        let service = TaskService::new(1, None, 200);
        let task = service.create(TaskKind::Archive, sources.len(), 0).await?;
        service.mark_running(&task.id).await?;
        let sources = sources.to_vec();
        let target = target.to_path_buf();
        let handle = Handle::current();
        tokio::task::spawn_blocking(move || {
            let mut progress = BlockingProgress::new(handle, service, task.id, None);
            archive_zip_sync(&sources, &target, &mut progress)
        })
        .await?
    }

    async fn run_extract_zip(
        source: &FsPath,
        output: &FsPath,
        limits: ExtractLimits,
    ) -> Result<u64, AppError> {
        let service = TaskService::new(1, None, 200);
        let task = service.create(TaskKind::Extract, 1, 0).await?;
        service.mark_running(&task.id).await?;
        let source = source.to_path_buf();
        let output = output.to_path_buf();
        let handle = Handle::current();
        tokio::task::spawn_blocking(move || {
            let mut progress = BlockingProgress::new(handle, service, task.id, None);
            extract_zip_sync(&source, &output, limits, &mut progress)
        })
        .await?
    }

    async fn run_copy_path(source: &FsPath, target: &FsPath) -> Result<u64, AppError> {
        let service = TaskService::new(1, None, 200);
        let task = service.create(TaskKind::Copy, 1, 0).await?;
        service.mark_running(&task.id).await?;
        let source = source.to_path_buf();
        let target = target.to_path_buf();
        let handle = Handle::current();
        tokio::task::spawn_blocking(move || {
            let mut progress = BlockingProgress::new(handle, service, task.id, None);
            copy_path_streaming(&source, &target, &mut progress)
        })
        .await?
    }

    fn try_create_file_symlink(source: &FsPath, link: &FsPath) -> bool {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(source, link).is_ok()
        }
        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_file(source, link).is_ok()
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = (source, link);
            false
        }
    }

    fn create_zip(path: &FsPath, files: &[(&str, &[u8])]) {
        let file = fs::File::create(path).unwrap();
        let mut writer = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        for (name, content) in files {
            writer.start_file(*name, options).unwrap();
            writer.write_all(content).unwrap();
        }
        writer.finish().unwrap();
    }

    fn temp_dir(prefix: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nonce}"))
    }
}
