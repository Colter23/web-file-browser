use std::{collections::HashMap, sync::Arc};

use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tokio::sync::{OwnedSemaphorePermit, RwLock, Semaphore};
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{TaskError, TaskKind, TaskState, TaskStatus},
};

#[derive(Clone)]
pub struct TaskService {
    tasks: Arc<RwLock<HashMap<String, TaskStatus>>>,
    run_slots: Arc<Semaphore>,
    speed_limit_bytes_per_sec: Option<u64>,
}

impl TaskService {
    pub fn new(max_concurrency: usize, speed_limit_bytes_per_sec: Option<u64>) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            run_slots: Arc::new(Semaphore::new(max_concurrency.max(1))),
            speed_limit_bytes_per_sec,
        }
    }

    pub async fn acquire_run_permit(&self) -> Result<OwnedSemaphorePermit, AppError> {
        self.run_slots
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| AppError::internal("任务调度器已关闭"))
    }

    pub fn speed_limit_bytes_per_sec(&self) -> Option<u64> {
        self.speed_limit_bytes_per_sec
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
        task.cancelled = true;
        if matches!(task.state, TaskState::Queued | TaskState::Running) {
            task.state = TaskState::Cancelled;
            task.finished_at = Some(now_string()?);
        }
        Ok(task.clone())
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

    pub async fn add_error(&self, id: &str, path: String, message: String) -> Result<(), AppError> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(id) {
            task.errors.push(TaskError { path, message });
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
        }
        Ok(())
    }

    pub async fn count_total(&self) -> usize {
        self.tasks.read().await.len()
    }

    pub async fn count_running(&self) -> usize {
        self.tasks
            .read()
            .await
            .values()
            .filter(|task| matches!(task.state, TaskState::Queued | TaskState::Running))
            .count()
    }
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

impl Default for TaskService {
    fn default() -> Self {
        Self::new(2, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{TaskKind, TaskState};

    #[tokio::test]
    async fn tracks_task_bytes_and_speed() {
        let service = TaskService::new(1, Some(1024));
        let task = service.create(TaskKind::Copy, 1, 2048).await.unwrap();
        service.mark_running(&task.id).await.unwrap();
        service.bytes_done(&task.id, 512).await.unwrap();
        let status = service.get(&task.id).await.unwrap();

        assert_eq!(status.processed_bytes, 512);
        assert!(status.speed_bytes_per_sec >= 0.0);
        assert_eq!(status.state, TaskState::Running);
        assert_eq!(service.speed_limit_bytes_per_sec(), Some(1024));
    }
}
