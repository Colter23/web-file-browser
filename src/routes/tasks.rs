use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use std::{
    fs,
    io::{Read, Write},
    path::{Path as FsPath, PathBuf},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};
use tokio::runtime::Handle;

use crate::{
    app::AppState,
    error::AppError,
    models::{
        DeleteTaskRequest, MoveEntryRequest, TaskKind, TaskPathRequest, TaskResponse, TaskStatus,
    },
    services::{
        file_ops,
        path_resolver::{
            self, ensure_folder, ensure_writable, join_virtual_path, normalize_child_name,
        },
    },
};

const COPY_BUFFER_SIZE: usize = 256 * 1024;

pub fn task_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/tasks", get(list_tasks))
        .route("/tasks/copy", post(create_copy_task))
        .route("/tasks/move", post(create_move_task))
        .route("/tasks/delete", post(create_delete_task))
        .route("/tasks/{id}", get(get_task))
        .route("/tasks/{id}/cancel", post(cancel_task))
}

async fn list_tasks(State(state): State<Arc<AppState>>) -> Json<Vec<TaskStatus>> {
    Json(state.tasks.list().await)
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
    let task = state
        .tasks
        .create(TaskKind::Copy, request.sources.len(), 0)
        .await?;
    let id = task.id.clone();
    spawn_copy_task(state, id.clone(), request.sources, target_path);
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
    let task = state
        .tasks
        .create(TaskKind::Move, request.sources.len(), 0)
        .await?;
    let id = task.id.clone();
    spawn_move_task(state, id.clone(), request.sources, target_path);
    Ok(Json(TaskResponse { id }))
}

async fn create_delete_task(
    State(state): State<Arc<AppState>>,
    Json(request): Json<DeleteTaskRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    if request.paths.is_empty() {
        return Err(AppError::bad_request("删除任务 paths 不能为空"));
    }
    let task = state
        .tasks
        .create(TaskKind::Delete, request.paths.len(), 0)
        .await?;
    let id = task.id.clone();
    spawn_delete_task(state, id.clone(), request.paths);
    Ok(Json(TaskResponse { id }))
}

fn spawn_copy_task(
    state: Arc<AppState>,
    task_id: String,
    sources: Vec<String>,
    target_path: String,
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
            let result = copy_one(
                state.clone(),
                task_id.clone(),
                source_path.clone(),
                target_path.clone(),
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
            let result = move_one(state.clone(), source_path.clone(), target_path.clone()).await;
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
            let result = delete_one(state.clone(), path.clone()).await;
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

async fn copy_one(
    state: Arc<AppState>,
    task_id: String,
    source_path: String,
    target_path: String,
) -> Result<u64, AppError> {
    let snapshot = state.mapping_store.snapshot().await;
    let source = path_resolver::resolve_existing(snapshot.clone(), source_path).await?;
    let target_parent = path_resolver::resolve_existing(snapshot, target_path).await?;
    ensure_writable(&target_parent.mapping)?;
    ensure_folder(&target_parent.real_path, &target_parent.virtual_path)?;
    let name = source
        .real_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| AppError::bad_request("源路径没有文件名"))
        .and_then(normalize_child_name)?;
    let target = target_parent.real_path.join(&name);
    if target.exists() {
        return Err(AppError::conflict(format!(
            "路径已存在: {}",
            join_virtual_path(&target_parent.virtual_path, &name)
        )));
    }
    let source_real_path = source.real_path.clone();
    let tasks = state.tasks.clone();
    let speed_limit = state.tasks.speed_limit_bytes_per_sec();
    let handle = Handle::current();
    let bytes = tokio::task::spawn_blocking(move || {
        let mut progress = CopyProgress::new(handle, tasks, task_id, speed_limit);
        copy_path_streaming(&source_real_path, &target, &mut progress)
    })
    .await??;
    state
        .audit
        .record("admin", "task.copy", Some(&source.virtual_path), None)
        .await?;
    Ok(bytes)
}

async fn move_one(
    state: Arc<AppState>,
    source_path: String,
    target_path: String,
) -> Result<(), AppError> {
    let snapshot = state.mapping_store.snapshot().await;
    let source = path_resolver::resolve_existing(snapshot.clone(), source_path.clone()).await?;
    let name = source
        .real_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| AppError::bad_request("源路径没有文件名"))
        .and_then(normalize_child_name)?;
    let target = join_virtual_path(&target_path, &name);
    file_ops::move_entry(
        snapshot,
        source_path,
        MoveEntryRequest {
            target_path: target,
        },
    )
    .await?;
    state
        .audit
        .record("admin", "task.move", Some(&source.virtual_path), None)
        .await?;
    Ok(())
}

async fn delete_one(state: Arc<AppState>, path: String) -> Result<u64, AppError> {
    let snapshot = state.mapping_store.snapshot().await;
    let target = file_ops::resolve_delete_target(snapshot, path).await?;
    let size = path_size(&target.real_path)?;
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
    Ok(size)
}

struct CopyProgress {
    handle: Handle,
    tasks: crate::services::tasks::TaskService,
    task_id: String,
    speed_limit_bytes_per_sec: Option<u64>,
    started: Instant,
    copied_bytes: u64,
}

impl CopyProgress {
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
            copied_bytes: 0,
        }
    }

    fn ensure_not_cancelled(&self) -> Result<(), AppError> {
        if self.handle.block_on(self.tasks.is_cancelled(&self.task_id)) {
            Err(AppError::conflict("任务已取消"))
        } else {
            Ok(())
        }
    }

    fn chunk_done(&mut self, bytes: u64) -> Result<(), AppError> {
        self.ensure_not_cancelled()?;
        self.copied_bytes = self.copied_bytes.saturating_add(bytes);
        self.handle
            .block_on(self.tasks.bytes_done(&self.task_id, bytes))?;
        if let Some(limit) = self.speed_limit_bytes_per_sec {
            let expected_elapsed = Duration::from_secs_f64(self.copied_bytes as f64 / limit as f64);
            let elapsed = self.started.elapsed();
            if expected_elapsed > elapsed {
                thread::sleep(expected_elapsed - elapsed);
            }
        }
        Ok(())
    }
}

fn copy_path_streaming(
    source: &FsPath,
    target: &FsPath,
    progress: &mut CopyProgress,
) -> Result<u64, AppError> {
    progress.ensure_not_cancelled()?;
    let metadata = fs::metadata(source)?;
    if metadata.is_dir() {
        copy_dir_streaming(source, target, progress)
    } else {
        copy_file_streaming(source, target, progress)
    }
}

fn copy_dir_streaming(
    source: &FsPath,
    target: &FsPath,
    progress: &mut CopyProgress,
) -> Result<u64, AppError> {
    fs::create_dir_all(target)?;
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
    progress: &mut CopyProgress,
) -> Result<u64, AppError> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut reader = fs::File::open(source)?;
    let mut writer = fs::File::create(target)?;
    let mut buffer = vec![0_u8; COPY_BUFFER_SIZE];
    let mut bytes = 0;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        writer.write_all(&buffer[..read])?;
        progress.chunk_done(read as u64)?;
        bytes += read as u64;
    }
    writer.flush()?;
    Ok(bytes)
}

fn path_size(path: &PathBuf) -> Result<u64, AppError> {
    let metadata = fs::metadata(path)?;
    if metadata.is_file() {
        return Ok(metadata.len());
    }
    let mut size = 0;
    for entry in fs::read_dir(path)? {
        size += path_size(&entry?.path())?;
    }
    Ok(size)
}
