use std::{
    collections::HashMap,
    sync::{
        Arc, RwLock as StdRwLock,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
};

use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tokio::sync::{OwnedSemaphorePermit, RwLock, Semaphore};
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{TaskError, TaskKind, TaskMetrics, TaskState, TaskStatus},
};

#[derive(Clone)]
pub struct TaskService {
    tasks: Arc<RwLock<HashMap<String, TaskStatus>>>,
    runtime: Arc<StdRwLock<Arc<TaskRuntime>>>,
    speed_limit_bytes_per_sec: Arc<AtomicU64>,
    history_limit: Arc<AtomicUsize>,
}

struct TaskRuntime {
    run_slots: Arc<Semaphore>,
}

impl TaskService {
    pub fn new(
        max_concurrency: usize,
        speed_limit_bytes_per_sec: Option<u64>,
        history_limit: usize,
    ) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            runtime: Arc::new(StdRwLock::new(Arc::new(TaskRuntime::new(max_concurrency)))),
            speed_limit_bytes_per_sec: Arc::new(AtomicU64::new(option_to_atomic(
                speed_limit_bytes_per_sec,
            ))),
            history_limit: Arc::new(AtomicUsize::new(history_limit.max(1))),
        }
    }

    pub fn update(
        &self,
        max_concurrency: usize,
        speed_limit_bytes_per_sec: Option<u64>,
        history_limit: usize,
    ) {
        let mut runtime = self
            .runtime
            .write()
            .unwrap_or_else(|error| error.into_inner());
        *runtime = Arc::new(TaskRuntime::new(max_concurrency));
        self.speed_limit_bytes_per_sec.store(
            option_to_atomic(speed_limit_bytes_per_sec),
            Ordering::Relaxed,
        );
        self.history_limit
            .store(history_limit.max(1), Ordering::Relaxed);
    }

    pub async fn acquire_run_permit(&self) -> Result<OwnedSemaphorePermit, AppError> {
        self.runtime()
            .run_slots
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| AppError::internal("任务调度器已关闭"))
    }

    pub fn speed_limit_bytes_per_sec(&self) -> Option<u64> {
        atomic_to_option(self.speed_limit_bytes_per_sec.load(Ordering::Relaxed))
    }

    pub async fn create(
        &self,
        kind: TaskKind,
        total_items: usize,
        total_bytes: u64,
    ) -> Result<TaskStatus, AppError> {
        let id = Uuid::new_v4().to_string();
        let now = now_string()?;
        let status = TaskStatus {
            id: id.clone(),
            kind,
            state: TaskState::Queued,
            progress: 0.0,
            processed_bytes: 0,
            total_bytes,
            speed_bytes_per_sec: 0.0,
            processed_items: 0,
            total_items,
            current_path: None,
            errors: Vec::new(),
            started_at: None,
            finished_at: None,
            created_at: now,
            cancelled: false,
        };
        self.tasks.write().await.insert(id, status.clone());
        Ok(status)
    }

    pub async fn list(&self) -> Vec<TaskStatus> {
        let mut tasks = self
            .tasks
            .read()
            .await
            .values()
            .cloned()
            .collect::<Vec<_>>();
        tasks.sort_by(|left, right| right.created_at.cmp(&left.created_at));
        tasks
    }

    pub async fn get(&self, id: &str) -> Result<TaskStatus, AppError> {
        self.tasks
            .read()
            .await
            .get(id)
            .cloned()
            .ok_or_else(|| AppError::not_found(format!("查无此任务: {id}")))
    }

    pub async fn cancel(&self, id: &str) -> Result<TaskStatus, AppError> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(id)
            .ok_or_else(|| AppError::not_found(format!("查无此任务: {id}")))?;
        if is_finished(task) {
            return Err(AppError::conflict("任务已结束，不能取消"));
        }
        task.cancelled = true;
        task.state = TaskState::Cancelled;
        task.finished_at = Some(now_string()?);
        task.current_path = None;
        let status = task.clone();
        prune_history(&mut tasks, self.history_limit());
        Ok(status)
    }

    pub async fn mark_running(&self, id: &str) -> Result<(), AppError> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(id)
            && !task.cancelled
        {
            task.state = TaskState::Running;
            task.started_at = Some(now_string()?);
        }
        Ok(())
    }

    pub async fn is_cancelled(&self, id: &str) -> bool {
        self.tasks
            .read()
            .await
            .get(id)
            .map(|task| task.cancelled)
            .unwrap_or(true)
    }

    pub async fn ensure_not_cancelled(&self, id: &str) -> Result<(), AppError> {
        if self.is_cancelled(id).await {
            Err(AppError::conflict("任务已取消"))
        } else {
            Ok(())
        }
    }

    pub async fn set_current_path(
        &self,
        id: &str,
        current_path: Option<String>,
    ) -> Result<(), AppError> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(id)
            && matches!(task.state, TaskState::Queued | TaskState::Running)
        {
            task.current_path = current_path;
        }
        Ok(())
    }

    pub async fn item_done(&self, id: &str, bytes: u64) -> Result<(), AppError> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(id) {
            task.processed_items = task.processed_items.saturating_add(1);
            task.processed_bytes = task.processed_bytes.saturating_add(bytes);
            task.progress = progress(task.processed_items, task.total_items);
            task.speed_bytes_per_sec = speed_bytes_per_sec(task);
        }
        Ok(())
    }

    pub async fn bytes_done(&self, id: &str, bytes: u64) -> Result<(), AppError> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(id) {
            task.processed_bytes = task.processed_bytes.saturating_add(bytes);
            task.speed_bytes_per_sec = speed_bytes_per_sec(task);
        }
        Ok(())
    }

    pub async fn add_error(&self, id: &str, path: String, error: AppError) -> Result<(), AppError> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(id) {
            task.errors.push(TaskError {
                path,
                code: error.code().to_string(),
                reason: error.reason().to_string(),
                message: error.to_string(),
                params: error.params().cloned(),
            });
        }
        Ok(())
    }

    pub async fn finish(&self, id: &str) -> Result<(), AppError> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(id) {
            if task.cancelled {
                task.state = TaskState::Cancelled;
            } else if task.errors.is_empty() {
                task.state = TaskState::Completed;
                task.progress = 1.0;
            } else {
                task.state = TaskState::Failed;
            }
            task.finished_at = Some(now_string()?);
            task.current_path = None;
        }
        prune_history(&mut tasks, self.history_limit());
        Ok(())
    }

    pub async fn metrics(&self) -> TaskMetrics {
        let tasks = self.tasks.read().await;
        let mut metrics = TaskMetrics {
            total: tasks.len(),
            ..TaskMetrics::default()
        };
        for task in tasks.values() {
            match task.state {
                TaskState::Queued => metrics.queued += 1,
                TaskState::Running => metrics.running += 1,
                TaskState::Completed => metrics.completed += 1,
                TaskState::Failed => metrics.failed += 1,
                TaskState::Cancelled => metrics.cancelled += 1,
            }
            metrics.errors_total = metrics.errors_total.saturating_add(task.errors.len());
            metrics.processed_bytes = metrics.processed_bytes.saturating_add(task.processed_bytes);
            if matches!(task.state, TaskState::Running) {
                metrics.current_speed_bytes_per_sec += task.speed_bytes_per_sec;
            }
        }
        metrics
    }

    pub async fn cleanup_finished(&self) -> usize {
        let mut tasks = self.tasks.write().await;
        let before = tasks.len();
        tasks.retain(|_, task| !is_finished(task));
        before.saturating_sub(tasks.len())
    }

    fn runtime(&self) -> Arc<TaskRuntime> {
        self.runtime
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    fn history_limit(&self) -> usize {
        self.history_limit.load(Ordering::Relaxed).max(1)
    }
}

impl TaskRuntime {
    fn new(max_concurrency: usize) -> Self {
        Self {
            run_slots: Arc::new(Semaphore::new(max_concurrency.max(1))),
        }
    }
}

fn prune_history(tasks: &mut HashMap<String, TaskStatus>, history_limit: usize) {
    let mut finished = tasks
        .values()
        .filter(|task| is_finished(task))
        .map(|task| {
            (
                task.id.clone(),
                task.finished_at
                    .clone()
                    .unwrap_or_else(|| task.created_at.clone()),
            )
        })
        .collect::<Vec<_>>();
    if finished.len() <= history_limit {
        return;
    }

    finished.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| right.0.cmp(&left.0)));
    for (id, _) in finished.into_iter().skip(history_limit) {
        tasks.remove(&id);
    }
}

fn is_finished(task: &TaskStatus) -> bool {
    matches!(
        task.state,
        TaskState::Completed | TaskState::Failed | TaskState::Cancelled
    )
}

fn progress(processed: usize, total: usize) -> f64 {
    if total == 0 {
        1.0
    } else {
        (processed as f64 / total as f64).clamp(0.0, 1.0)
    }
}

fn speed_bytes_per_sec(task: &TaskStatus) -> f64 {
    let Some(started_at) = &task.started_at else {
        return 0.0;
    };
    let Ok(started_at) = OffsetDateTime::parse(started_at, &Rfc3339) else {
        return 0.0;
    };
    let elapsed_ms = (OffsetDateTime::now_utc() - started_at)
        .whole_milliseconds()
        .max(1) as f64;
    task.processed_bytes as f64 / (elapsed_ms / 1000.0)
}

fn now_string() -> Result<String, AppError> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| AppError::internal(format!("生成任务时间失败: {error}")))
}

fn option_to_atomic(value: Option<u64>) -> u64 {
    value.unwrap_or(0)
}

fn atomic_to_option(value: u64) -> Option<u64> {
    if value == 0 { None } else { Some(value) }
}

impl Default for TaskService {
    fn default() -> Self {
        Self::new(2, None, 200)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{TaskKind, TaskState};

    #[tokio::test]
    async fn tracks_task_bytes_and_speed() {
        let service = TaskService::new(1, Some(1024), 200);
        let task = service.create(TaskKind::Copy, 1, 2048).await.unwrap();
        service.mark_running(&task.id).await.unwrap();
        service.bytes_done(&task.id, 512).await.unwrap();
        let status = service.get(&task.id).await.unwrap();

        assert_eq!(status.processed_bytes, 512);
        assert!(status.speed_bytes_per_sec >= 0.0);
        assert_eq!(status.state, TaskState::Running);
        assert_eq!(service.speed_limit_bytes_per_sec(), Some(1024));
    }

    #[tokio::test]
    async fn prunes_old_finished_tasks() {
        let service = TaskService::new(1, None, 2);
        let first = service.create(TaskKind::Copy, 1, 0).await.unwrap();
        service.finish(&first.id).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(2)).await;

        let second = service.create(TaskKind::Move, 1, 0).await.unwrap();
        service.finish(&second.id).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(2)).await;

        let third = service.create(TaskKind::Delete, 1, 0).await.unwrap();
        service.finish(&third.id).await.unwrap();

        let ids = service
            .list()
            .await
            .into_iter()
            .map(|task| task.id)
            .collect::<Vec<_>>();

        assert_eq!(ids.len(), 2);
        assert!(!ids.contains(&first.id));
        assert!(ids.contains(&second.id));
        assert!(ids.contains(&third.id));
    }

    #[tokio::test]
    async fn keeps_running_tasks_when_pruning_history() {
        let service = TaskService::new(1, None, 1);
        let old = service.create(TaskKind::Copy, 1, 0).await.unwrap();
        service.finish(&old.id).await.unwrap();

        let running = service.create(TaskKind::Move, 1, 0).await.unwrap();
        service.mark_running(&running.id).await.unwrap();

        let latest = service.create(TaskKind::Delete, 1, 0).await.unwrap();
        service.finish(&latest.id).await.unwrap();

        let tasks = service.list().await;
        let ids = tasks.iter().map(|task| task.id.clone()).collect::<Vec<_>>();

        assert_eq!(tasks.len(), 2);
        assert!(!ids.contains(&old.id));
        assert!(ids.contains(&running.id));
        assert!(ids.contains(&latest.id));
    }

    #[tokio::test]
    async fn metrics_counts_task_states_and_errors() {
        let service = TaskService::new(1, None, 10);
        let queued = service.create(TaskKind::Copy, 1, 0).await.unwrap();
        let running = service.create(TaskKind::Move, 1, 0).await.unwrap();
        service.mark_running(&running.id).await.unwrap();
        service.bytes_done(&running.id, 128).await.unwrap();
        let failed = service.create(TaskKind::Delete, 1, 0).await.unwrap();
        service
            .add_error(
                &failed.id,
                "/repo/a.txt".to_string(),
                AppError::bad_request("失败").with_reason("TEST_ERROR"),
            )
            .await
            .unwrap();
        service.finish(&failed.id).await.unwrap();
        let cancelled = service.create(TaskKind::Archive, 1, 0).await.unwrap();
        service.cancel(&cancelled.id).await.unwrap();

        let metrics = service.metrics().await;

        assert_eq!(metrics.total, 4);
        assert_eq!(metrics.queued, 1);
        assert_eq!(metrics.running, 1);
        assert_eq!(metrics.failed, 1);
        assert_eq!(metrics.cancelled, 1);
        assert_eq!(metrics.errors_total, 1);
        assert_eq!(metrics.processed_bytes, 128);
        assert!(metrics.current_speed_bytes_per_sec >= 0.0);
        assert_eq!(queued.kind, TaskKind::Copy);
    }

    #[tokio::test]
    async fn ensure_not_cancelled_rejects_cancelled_task() {
        let service = TaskService::new(1, None, 10);
        let task = service.create(TaskKind::Move, 1, 0).await.unwrap();

        service.ensure_not_cancelled(&task.id).await.unwrap();
        service.cancel(&task.id).await.unwrap();

        let error = service.ensure_not_cancelled(&task.id).await.unwrap_err();
        assert!(matches!(error, AppError::Conflict(_)));
    }

    #[tokio::test]
    async fn cancel_rejects_finished_task_without_changing_status() {
        let service = TaskService::new(1, None, 10);
        let task = service.create(TaskKind::Copy, 1, 0).await.unwrap();
        service.finish(&task.id).await.unwrap();

        let error = service.cancel(&task.id).await.unwrap_err();
        let status = service.get(&task.id).await.unwrap();

        assert!(matches!(error, AppError::Conflict(_)));
        assert_eq!(status.state, TaskState::Completed);
        assert!(!status.cancelled);
    }

    #[tokio::test]
    async fn tracks_current_path_and_clears_when_finished() {
        let service = TaskService::new(1, None, 10);
        let task = service.create(TaskKind::Archive, 1, 0).await.unwrap();
        service.mark_running(&task.id).await.unwrap();
        service
            .set_current_path(&task.id, Some("/repo/file.txt".to_string()))
            .await
            .unwrap();

        let running = service.get(&task.id).await.unwrap();
        assert_eq!(running.current_path.as_deref(), Some("/repo/file.txt"));

        service.finish(&task.id).await.unwrap();
        let finished = service.get(&task.id).await.unwrap();
        assert_eq!(finished.current_path, None);
    }

    #[tokio::test]
    async fn explicit_cleanup_removes_finished_tasks_only() {
        let service = TaskService::new(1, None, 10);
        let completed = service.create(TaskKind::Copy, 1, 0).await.unwrap();
        service.finish(&completed.id).await.unwrap();
        let running = service.create(TaskKind::Move, 1, 0).await.unwrap();
        service.mark_running(&running.id).await.unwrap();
        let queued = service.create(TaskKind::Delete, 1, 0).await.unwrap();

        let removed = service.cleanup_finished().await;
        let ids = service
            .list()
            .await
            .into_iter()
            .map(|task| task.id)
            .collect::<Vec<_>>();

        assert_eq!(removed, 1);
        assert!(!ids.contains(&completed.id));
        assert!(ids.contains(&running.id));
        assert!(ids.contains(&queued.id));
    }
}
