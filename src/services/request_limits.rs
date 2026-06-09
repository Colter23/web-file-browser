use std::{collections::HashMap, sync::Arc};

use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore};

use crate::error::AppError;

#[derive(Clone)]
pub struct RequestLimits {
    dir_scan: Arc<Semaphore>,
    transfer: Arc<Semaphore>,
    per_ip_limit: usize,
    per_ip: Arc<Mutex<HashMap<String, Arc<Semaphore>>>>,
}

impl RequestLimits {
    pub fn new(dir_scan: usize, transfer: usize, per_ip_limit: usize) -> Self {
        Self {
            dir_scan: Arc::new(Semaphore::new(dir_scan.max(1))),
            transfer: Arc::new(Semaphore::new(transfer.max(1))),
            per_ip_limit: per_ip_limit.max(1),
            per_ip: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn acquire_dir_scan(&self) -> Result<OwnedSemaphorePermit, AppError> {
        self.dir_scan
            .clone()
            .try_acquire_owned()
            .map_err(|_| AppError::too_many_requests("目录扫描并发过高，请稍后重试"))
    }

    pub fn acquire_transfer(&self) -> Result<OwnedSemaphorePermit, AppError> {
        self.transfer
            .clone()
            .try_acquire_owned()
            .map_err(|_| AppError::too_many_requests("文件传输并发过高，请稍后重试"))
    }

    pub async fn acquire_ip(&self, ip: String) -> Result<OwnedSemaphorePermit, AppError> {
        let semaphore = {
            let mut per_ip = self.per_ip.lock().await;
            per_ip
                .entry(ip)
                .or_insert_with(|| Arc::new(Semaphore::new(self.per_ip_limit)))
                .clone()
        };
        semaphore
            .try_acquire_owned()
            .map_err(|_| AppError::too_many_requests("当前 IP 并发请求过高，请稍后重试"))
    }
}
