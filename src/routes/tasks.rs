use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use serde::Serialize;
use std::{collections::HashSet, sync::Arc};
use tokio::runtime::Handle;

use crate::{
    app::AppState,
    error::AppError,
    models::{
        ArchiveTaskRequest, ConflictPolicy, DeleteTaskRequest, ExtractTaskRequest,
        MoveEntryRequest, TaskKind, TaskPathRequest, TaskResponse, TaskStatus,
    },
    services::{
        conflict, file_ops,
        path_resolver::{
            self, MappingSnapshot, ensure_file, ensure_folder, ensure_writable, join_virtual_path,
            normalize_child_name,
        },
        task_io::{
            ArchiveSource, BlockingProgress, ExtractLimits, archive_output_name,
            archive_sources_sync, cleanup_path, copy_path_streaming, extract_archive_sync,
            extract_output_folder_name, finalize_temp_path, temp_sibling_path,
        },
    },
};

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
    let runtime = state.settings.runtime().await;
    let policy = request.conflict_policy.unwrap_or(runtime.conflict_policy);
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
    let runtime = state.settings.runtime().await;
    let policy = request.conflict_policy.unwrap_or(runtime.conflict_policy);
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
    spawn_delete_task(state, id.clone(), request.paths, request.permanent);
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
    let runtime = state.settings.runtime().await;
    let policy = request.conflict_policy.unwrap_or(runtime.conflict_policy);
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
    let runtime = state.settings.runtime().await;
    let policy = request.conflict_policy.unwrap_or(runtime.conflict_policy);
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

fn spawn_delete_task(state: Arc<AppState>, task_id: String, paths: Vec<String>, permanent: bool) {
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
            let result = delete_one(state.clone(), task_id.clone(), path.clone(), permanent).await;
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
    let runtime = state.settings.runtime().await;
    let limits = ExtractLimits {
        max_bytes: runtime.max_extract_bytes,
        max_files: runtime.max_extract_files,
        max_depth: runtime.max_extract_depth,
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

async fn delete_one(
    state: Arc<AppState>,
    task_id: String,
    path: String,
    permanent: bool,
) -> Result<u64, AppError> {
    let snapshot = state.mapping_store.snapshot().await;
    if permanent {
        state.tasks.ensure_not_cancelled(&task_id).await?;
        let response = file_ops::permanent_delete(snapshot, path).await?;
        state
            .audit
            .record("admin", "task.delete.permanent", Some(&response.path), None)
            .await?;
        index_remove_ignore(&state, &response.path).await;
        return Ok(0);
    }

    let target = file_ops::resolve_delete_target(snapshot, path).await?;
    state.tasks.ensure_not_cancelled(&task_id).await?;
    let original_virtual_path = target.virtual_path.clone();
    let original_real_path = target.real_path.to_string_lossy().to_string();
    state
        .trash
        .move_to_trash(
            target.real_path,
            target.mount_root,
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
