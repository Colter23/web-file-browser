use std::{
    collections::HashMap,
    sync::{Arc, RwLock as StdRwLock},
};

use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore};

use crate::{error::AppError, models::RequestLimitMetrics};

#[derive(Clone)]
pub struct RequestLimits {
    inner: Arc<StdRwLock<Arc<RequestLimitState>>>,
}

struct RequestLimitState {
    dir_scan: Arc<Semaphore>,
    transfer: Arc<Semaphore>,
    dir_scan_limit: usize,
    transfer_limit: usize,
    per_ip_limit: usize,
    per_ip: Arc<Mutex<HashMap<String, Arc<Semaphore>>>>,
}

impl RequestLimits {
    pub fn new(dir_scan: usize, transfer: usize, per_ip_limit: usize) -> Self {
        Self {
            inner: Arc::new(StdRwLock::new(Arc::new(RequestLimitState::new(
                dir_scan,
                transfer,
                per_ip_limit,
            )))),
        }
    }

    pub fn update(&self, dir_scan: usize, transfer: usize, per_ip_limit: usize) {
        let mut inner = self
            .inner
            .write()
            .unwrap_or_else(|error| error.into_inner());
        *inner = Arc::new(RequestLimitState::new(dir_scan, transfer, per_ip_limit));
    }

    pub fn acquire_dir_scan(&self) -> Result<OwnedSemaphorePermit, AppError> {
        self.current().acquire_dir_scan()
    }

    pub fn acquire_transfer(&self) -> Result<OwnedSemaphorePermit, AppError> {
        self.current().acquire_transfer()
    }

    pub async fn acquire_ip(&self, ip: String) -> Result<OwnedSemaphorePermit, AppError> {
        self.current().acquire_ip(ip).await
    }

    pub async fn metrics(&self) -> RequestLimitMetrics {
        self.current().metrics().await
    }

    fn current(&self) -> Arc<RequestLimitState> {
        self.inner
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }
}

impl RequestLimitState {
    fn new(dir_scan: usize, transfer: usize, per_ip_limit: usize) -> Self {
        let dir_scan_limit = dir_scan.max(1);
        let transfer_limit = transfer.max(1);
        let per_ip_limit = per_ip_limit.max(1);
        Self {
            dir_scan: Arc::new(Semaphore::new(dir_scan_limit)),
            transfer: Arc::new(Semaphore::new(transfer_limit)),
            dir_scan_limit,
            transfer_limit,
            per_ip_limit,
            per_ip: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn acquire_dir_scan(&self) -> Result<OwnedSemaphorePermit, AppError> {
        self.dir_scan.clone().try_acquire_owned().map_err(|_| {
            AppError::too_many_requests("目录扫描并发过高，请稍后重试")
                .with_reason("DIR_SCAN_CONCURRENCY_LIMITED")
                .with_param("limit", self.dir_scan_limit)
        })
    }

    fn acquire_transfer(&self) -> Result<OwnedSemaphorePermit, AppError> {
        self.transfer.clone().try_acquire_owned().map_err(|_| {
            AppError::too_many_requests("文件传输并发过高，请稍后重试")
                .with_reason("TRANSFER_CONCURRENCY_LIMITED")
                .with_param("limit", self.transfer_limit)
        })
    }

    async fn acquire_ip(&self, ip: String) -> Result<OwnedSemaphorePermit, AppError> {
        let mut per_ip = self.per_ip.lock().await;
        prune_idle_ip_entries(&mut per_ip, self.per_ip_limit);
        let semaphore = per_ip
            .entry(ip)
            .or_insert_with(|| Arc::new(Semaphore::new(self.per_ip_limit)))
            .clone();
        semaphore.try_acquire_owned().map_err(|_| {
            AppError::too_many_requests("当前 IP 并发请求过高，请稍后重试")
                .with_reason("IP_CONCURRENCY_LIMITED")
                .with_param("limit", self.per_ip_limit)
        })
    }

    async fn metrics(&self) -> RequestLimitMetrics {
        let mut per_ip = self.per_ip.lock().await;
        prune_idle_ip_entries(&mut per_ip, self.per_ip_limit);
        let active_ip_requests = per_ip
            .values()
            .map(|semaphore| {
                self.per_ip_limit
                    .saturating_sub(semaphore.available_permits())
            })
            .sum();
        RequestLimitMetrics {
            dir_scan_limit: self.dir_scan_limit,
            active_dir_scans: self
                .dir_scan_limit
                .saturating_sub(self.dir_scan.available_permits()),
            transfer_limit: self.transfer_limit,
            active_transfers: self
                .transfer_limit
                .saturating_sub(self.transfer.available_permits()),
            ip_limit: self.per_ip_limit,
            tracked_ips: per_ip.len(),
            active_ip_requests,
        }
    }
}

fn prune_idle_ip_entries(per_ip: &mut HashMap<String, Arc<Semaphore>>, per_ip_limit: usize) {
    per_ip.retain(|_, semaphore| semaphore.available_permits() < per_ip_limit);
}

#[cfg(test)]
mod tests {
    use super::RequestLimits;
    use crate::error::AppError;

    #[tokio::test]
    async fn metrics_reports_active_permits() {
        let limits = RequestLimits::new(2, 3, 4);
        let dir_permit = limits.acquire_dir_scan().unwrap();
        let transfer_permit = limits.acquire_transfer().unwrap();
        let ip_permit = limits.acquire_ip("127.0.0.1".to_string()).await.unwrap();

        let metrics = limits.metrics().await;

        assert_eq!(metrics.dir_scan_limit, 2);
        assert_eq!(metrics.active_dir_scans, 1);
        assert_eq!(metrics.transfer_limit, 3);
        assert_eq!(metrics.active_transfers, 1);
        assert_eq!(metrics.ip_limit, 4);
        assert_eq!(metrics.tracked_ips, 1);
        assert_eq!(metrics.active_ip_requests, 1);

        drop(dir_permit);
        drop(transfer_permit);
        drop(ip_permit);

        let metrics = limits.metrics().await;
        assert_eq!(metrics.active_dir_scans, 0);
        assert_eq!(metrics.active_transfers, 0);
        assert_eq!(metrics.active_ip_requests, 0);
        assert_eq!(metrics.tracked_ips, 0);
    }

    #[tokio::test]
    async fn prunes_idle_ip_entries_before_tracking_new_ip() {
        let limits = RequestLimits::new(2, 2, 1);
        let first = limits.acquire_ip("192.168.1.10".to_string()).await.unwrap();
        assert_eq!(limits.metrics().await.tracked_ips, 1);

        drop(first);
        let second = limits.acquire_ip("192.168.1.11".to_string()).await.unwrap();
        let metrics = limits.metrics().await;

        assert_eq!(metrics.tracked_ips, 1);
        assert_eq!(metrics.active_ip_requests, 1);

        drop(second);
    }

    #[tokio::test]
    async fn rejects_when_concurrency_limits_are_full() {
        let limits = RequestLimits::new(1, 1, 1);
        let dir_permit = limits.acquire_dir_scan().unwrap();
        let transfer_permit = limits.acquire_transfer().unwrap();
        let ip_permit = limits.acquire_ip("127.0.0.1".to_string()).await.unwrap();

        let dir_error = limits.acquire_dir_scan().unwrap_err();
        let transfer_error = limits.acquire_transfer().unwrap_err();
        let ip_error = limits
            .acquire_ip("127.0.0.1".to_string())
            .await
            .unwrap_err();

        assert!(matches!(dir_error, AppError::TooManyRequests(_)));
        assert!(matches!(transfer_error, AppError::TooManyRequests(_)));
        assert!(matches!(ip_error, AppError::TooManyRequests(_)));

        drop(dir_permit);
        drop(transfer_permit);
        drop(ip_permit);

        assert!(limits.acquire_dir_scan().is_ok());
        assert!(limits.acquire_transfer().is_ok());
        assert!(limits.acquire_ip("127.0.0.1".to_string()).await.is_ok());
    }
}
