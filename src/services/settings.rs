use std::{
    collections::BTreeSet,
    env,
    path::{Path, PathBuf},
    sync::Arc,
};

use tokio::{fs, sync::RwLock};
use uuid::Uuid;

use crate::{
    config::{
        AppConfig, ArchiveConfigFile, AuditConfigFile, EditorConfigFile, IndexConfigFile,
        LimitsConfigFile, RuntimeConfigFile, TaskConfigFile, TrashConfigFile,
        normalize_extension_values, normalize_mime_values,
    },
    error::AppError,
    models::{RuntimeSettings, RuntimeSettingsPatch, SettingsResponse, StartupSettings},
};

const RESTART_REQUIRED_FIELDS: &[&str] = &[
    "startup.bindAddress",
    "startup.port",
    "startup.staticDir",
    "startup.corsAllowedOrigins",
    "startup.trustProxyHeaders",
    "startup.configFile",
    "startup.authFile",
    "startup.mappingFile",
    "startup.favoritesFile",
    "startup.trashDir",
    "startup.auditFile",
    "startup.indexRebuildOnStartup",
];

const RUNTIME_ENV_FIELDS: &[(&str, &str)] = &[
    ("runtime.maxEditBytes", "WEB_FILE_BROWSER_MAX_EDIT_BYTES"),
    (
        "runtime.editableExtensions",
        "WEB_FILE_BROWSER_EDITABLE_EXTENSIONS",
    ),
    (
        "runtime.editableMimeTypes",
        "WEB_FILE_BROWSER_EDITABLE_MIME_TYPES",
    ),
    (
        "runtime.maxUploadBytes",
        "WEB_FILE_BROWSER_MAX_UPLOAD_BYTES",
    ),
    (
        "runtime.maxDirPageSize",
        "WEB_FILE_BROWSER_MAX_DIR_PAGE_SIZE",
    ),
    (
        "runtime.maxDirConcurrency",
        "WEB_FILE_BROWSER_MAX_DIR_CONCURRENCY",
    ),
    (
        "runtime.maxTransferConcurrency",
        "WEB_FILE_BROWSER_MAX_TRANSFER_CONCURRENCY",
    ),
    (
        "runtime.maxIpConcurrency",
        "WEB_FILE_BROWSER_MAX_IP_CONCURRENCY",
    ),
    (
        "runtime.maxTaskConcurrency",
        "WEB_FILE_BROWSER_MAX_TASK_CONCURRENCY",
    ),
    (
        "runtime.taskHistoryLimit",
        "WEB_FILE_BROWSER_TASK_HISTORY_LIMIT",
    ),
    (
        "runtime.taskSpeedLimitBytesPerSec",
        "WEB_FILE_BROWSER_TASK_SPEED_LIMIT_BYTES_PER_SEC",
    ),
    (
        "runtime.maxExtractBytes",
        "WEB_FILE_BROWSER_MAX_EXTRACT_BYTES",
    ),
    (
        "runtime.maxExtractFiles",
        "WEB_FILE_BROWSER_MAX_EXTRACT_FILES",
    ),
    (
        "runtime.maxExtractDepth",
        "WEB_FILE_BROWSER_MAX_EXTRACT_DEPTH",
    ),
    ("runtime.indexEnabled", "WEB_FILE_BROWSER_INDEX_ENABLED"),
    (
        "runtime.indexScanDelayMs",
        "WEB_FILE_BROWSER_INDEX_SCAN_DELAY_MS",
    ),
    ("runtime.auditMaxBytes", "WEB_FILE_BROWSER_AUDIT_MAX_BYTES"),
    (
        "runtime.auditRetentionFiles",
        "WEB_FILE_BROWSER_AUDIT_RETENTION_FILES",
    ),
    (
        "runtime.trashRetentionDays",
        "WEB_FILE_BROWSER_TRASH_RETENTION_DAYS",
    ),
    ("runtime.trashMaxBytes", "WEB_FILE_BROWSER_TRASH_MAX_BYTES"),
    ("runtime.conflictPolicy", "WEB_FILE_BROWSER_CONFLICT_POLICY"),
];

const STARTUP_ENV_FIELDS: &[(&str, &[&str])] = &[
    ("startup.bindAddress", &["WEB_FILE_BROWSER_BIND"]),
    ("startup.port", &["PORT"]),
    ("startup.staticDir", &["WEB_FILE_BROWSER_STATIC_DIR"]),
    (
        "startup.corsAllowedOrigins",
        &["WEB_FILE_BROWSER_CORS_ORIGINS"],
    ),
    (
        "startup.trustProxyHeaders",
        &["WEB_FILE_BROWSER_TRUST_PROXY_HEADERS"],
    ),
    (
        "startup.configFile",
        &["WEB_FILE_BROWSER_CONFIG_FILE", "WEB_FILE_BROWSER_CONFIG"],
    ),
    ("startup.authFile", &["WEB_FILE_BROWSER_AUTH_FILE"]),
    ("startup.mappingFile", &["WEB_FILE_BROWSER_MAPPING_FILE"]),
    (
        "startup.favoritesFile",
        &["WEB_FILE_BROWSER_FAVORITES_FILE"],
    ),
    ("startup.trashDir", &["WEB_FILE_BROWSER_TRASH_DIR"]),
    ("startup.auditFile", &["WEB_FILE_BROWSER_AUDIT_FILE"]),
    (
        "startup.indexRebuildOnStartup",
        &["WEB_FILE_BROWSER_INDEX_REBUILD_ON_STARTUP"],
    ),
];

#[derive(Clone)]
pub struct SettingsService {
    config_file: Arc<PathBuf>,
    runtime: Arc<RwLock<RuntimeSettings>>,
    startup: Arc<StartupSettings>,
    env_locked: Arc<BTreeSet<String>>,
    restart_required_fields: Arc<Vec<String>>,
}

impl SettingsService {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            config_file: Arc::new(config.config_file.clone()),
            runtime: Arc::new(RwLock::new(config.runtime_settings())),
            startup: Arc::new(config.startup_settings()),
            env_locked: Arc::new(env_locked_fields()),
            restart_required_fields: Arc::new(
                RESTART_REQUIRED_FIELDS
                    .iter()
                    .map(|field| (*field).to_string())
                    .collect(),
            ),
        }
    }

    pub async fn runtime(&self) -> RuntimeSettings {
        self.runtime.read().await.clone()
    }

    pub fn startup(&self) -> StartupSettings {
        (*self.startup).clone()
    }

    pub async fn response(&self, auth_configured: bool) -> SettingsResponse {
        SettingsResponse {
            runtime: self.runtime().await,
            startup: self.startup(),
            auth_configured,
            env_locked: self.env_locked.iter().cloned().collect(),
            restart_required_fields: self.restart_required_fields.as_ref().clone(),
        }
    }

    pub async fn patch_runtime(
        &self,
        patch: RuntimeSettingsPatch,
    ) -> Result<RuntimeSettings, AppError> {
        self.ensure_not_env_locked(&patch)?;
        let mut next = self.runtime().await;
        apply_patch(&mut next, patch)?;
        self.write_runtime_config(&next).await?;
        *self.runtime.write().await = next.clone();
        Ok(next)
    }

    pub async fn reload_runtime(&self) -> Result<RuntimeSettings, AppError> {
        let config = AppConfig::load_from_file(self.config_file.as_ref().clone())?;
        let next = config.runtime_settings();
        *self.runtime.write().await = next.clone();
        Ok(next)
    }

    fn ensure_not_env_locked(&self, patch: &RuntimeSettingsPatch) -> Result<(), AppError> {
        for field in touched_runtime_fields(patch) {
            if self.env_locked.contains(field) {
                return Err(AppError::conflict(format!(
                    "配置 {field} 由环境变量控制，请删除环境变量后重启"
                )));
            }
        }
        Ok(())
    }

    async fn write_runtime_config(&self, runtime: &RuntimeSettings) -> Result<(), AppError> {
        let mut config = RuntimeConfigFile::read(&self.config_file)?;
        fill_runtime_config(&mut config, runtime);
        write_config_atomic(&self.config_file, &config).await
    }
}

fn apply_patch(
    settings: &mut RuntimeSettings,
    patch: RuntimeSettingsPatch,
) -> Result<(), AppError> {
    if let Some(value) = patch.max_edit_bytes {
        settings.max_edit_bytes = positive_u64(value, "最大编辑大小")?;
    }
    if let Some(value) = patch.editable_extensions {
        settings.editable_extensions = normalize_extension_values(value);
    }
    if let Some(value) = patch.editable_mime_types {
        settings.editable_mime_types = normalize_mime_values(value);
    }
    if let Some(value) = patch.max_upload_bytes {
        settings.max_upload_bytes = positive_optional_u64(value, "最大上传大小")?;
    }
    if let Some(value) = patch.max_dir_page_size {
        settings.max_dir_page_size = positive_usize(value, "目录分页上限")?;
    }
    if let Some(value) = patch.max_dir_concurrency {
        settings.max_dir_concurrency = positive_usize(value, "目录扫描并发数")?;
    }
    if let Some(value) = patch.max_transfer_concurrency {
        settings.max_transfer_concurrency = positive_usize(value, "文件传输并发数")?;
    }
    if let Some(value) = patch.max_ip_concurrency {
        settings.max_ip_concurrency = positive_usize(value, "单 IP 并发数")?;
    }
    if let Some(value) = patch.max_task_concurrency {
        settings.max_task_concurrency = positive_usize(value, "后台任务并发数")?;
    }
    if let Some(value) = patch.task_history_limit {
        settings.task_history_limit = positive_usize(value, "任务历史保留数量")?;
    }
    if let Some(value) = patch.task_speed_limit_bytes_per_sec {
        settings.task_speed_limit_bytes_per_sec = positive_optional_u64(value, "任务限速")?;
    }
    if let Some(value) = patch.max_extract_bytes {
        settings.max_extract_bytes = positive_optional_u64(value, "最大解压字节数")?;
    }
    if let Some(value) = patch.max_extract_files {
        settings.max_extract_files = positive_optional_usize(value, "最大解压文件数")?;
    }
    if let Some(value) = patch.max_extract_depth {
        settings.max_extract_depth = positive_usize(value, "最大解压路径深度")?;
    }
    if let Some(value) = patch.index_enabled {
        settings.index_enabled = value;
    }
    if let Some(value) = patch.index_scan_delay_ms {
        settings.index_scan_delay_ms = value;
    }
    if let Some(value) = patch.audit_max_bytes {
        settings.audit_max_bytes = positive_optional_u64(value, "审计日志轮转大小")?;
    }
    if let Some(value) = patch.audit_retention_files {
        settings.audit_retention_files = value;
    }
    if let Some(value) = patch.trash_retention_days {
        settings.trash_retention_days = positive_optional_u64(value, "回收站保留天数")?;
    }
    if let Some(value) = patch.trash_max_bytes {
        settings.trash_max_bytes = positive_optional_u64(value, "回收站容量上限")?;
    }
    if let Some(value) = patch.conflict_policy {
        settings.conflict_policy = value;
    }
    Ok(())
}

fn fill_runtime_config(config: &mut RuntimeConfigFile, runtime: &RuntimeSettings) {
    config.limits = Some(LimitsConfigFile {
        max_upload_bytes: Some(runtime.max_upload_bytes),
        max_dir_page_size: Some(runtime.max_dir_page_size),
        max_dir_concurrency: Some(runtime.max_dir_concurrency),
        max_transfer_concurrency: Some(runtime.max_transfer_concurrency),
        max_ip_concurrency: Some(runtime.max_ip_concurrency),
    });
    config.editor = Some(EditorConfigFile {
        max_edit_bytes: Some(runtime.max_edit_bytes),
        editable_extensions: Some(runtime.editable_extensions.clone()),
        editable_mime_types: Some(runtime.editable_mime_types.clone()),
    });
    config.tasks = Some(TaskConfigFile {
        max_concurrency: Some(runtime.max_task_concurrency),
        history_limit: Some(runtime.task_history_limit),
        speed_limit_bytes_per_sec: Some(runtime.task_speed_limit_bytes_per_sec),
    });
    config.archive = Some(ArchiveConfigFile {
        max_extract_bytes: Some(runtime.max_extract_bytes),
        max_extract_files: Some(runtime.max_extract_files),
        max_extract_depth: Some(runtime.max_extract_depth),
    });
    let rebuild_on_startup = config
        .index
        .as_ref()
        .and_then(|index| index.rebuild_on_startup);
    config.index = Some(IndexConfigFile {
        enabled: Some(runtime.index_enabled),
        rebuild_on_startup,
        scan_delay_ms: Some(runtime.index_scan_delay_ms),
    });
    config.audit = Some(AuditConfigFile {
        max_bytes: Some(runtime.audit_max_bytes),
        retention_files: Some(runtime.audit_retention_files),
    });
    config.trash = Some(TrashConfigFile {
        retention_days: Some(runtime.trash_retention_days),
        max_bytes: Some(runtime.trash_max_bytes),
    });
    config.conflict_policy = Some(runtime.conflict_policy);
}

async fn write_config_atomic(path: &Path, config: &RuntimeConfigFile) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }

    let bytes = serde_json::to_vec_pretty(config)?;
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("config.json");
    let temp_path = path.with_file_name(format!(".{file_name}.{}.tmp", Uuid::new_v4()));
    fs::write(&temp_path, bytes).await?;

    match fs::rename(&temp_path, path).await {
        Ok(()) => Ok(()),
        Err(_error) if cfg!(windows) && path.exists() => {
            fs::remove_file(path).await?;
            if let Err(rename_error) = fs::rename(&temp_path, path).await {
                let _ = fs::remove_file(&temp_path).await;
                Err(rename_error.into())
            } else {
                Ok(())
            }
        }
        Err(error) => {
            let _ = fs::remove_file(&temp_path).await;
            Err(error.into())
        }
    }
}

fn touched_runtime_fields(patch: &RuntimeSettingsPatch) -> Vec<&'static str> {
    let mut fields = Vec::new();
    push_if(
        &mut fields,
        patch.max_edit_bytes.is_some(),
        "runtime.maxEditBytes",
    );
    push_if(
        &mut fields,
        patch.editable_extensions.is_some(),
        "runtime.editableExtensions",
    );
    push_if(
        &mut fields,
        patch.editable_mime_types.is_some(),
        "runtime.editableMimeTypes",
    );
    push_if(
        &mut fields,
        patch.max_upload_bytes.is_some(),
        "runtime.maxUploadBytes",
    );
    push_if(
        &mut fields,
        patch.max_dir_page_size.is_some(),
        "runtime.maxDirPageSize",
    );
    push_if(
        &mut fields,
        patch.max_dir_concurrency.is_some(),
        "runtime.maxDirConcurrency",
    );
    push_if(
        &mut fields,
        patch.max_transfer_concurrency.is_some(),
        "runtime.maxTransferConcurrency",
    );
    push_if(
        &mut fields,
        patch.max_ip_concurrency.is_some(),
        "runtime.maxIpConcurrency",
    );
    push_if(
        &mut fields,
        patch.max_task_concurrency.is_some(),
        "runtime.maxTaskConcurrency",
    );
    push_if(
        &mut fields,
        patch.task_history_limit.is_some(),
        "runtime.taskHistoryLimit",
    );
    push_if(
        &mut fields,
        patch.task_speed_limit_bytes_per_sec.is_some(),
        "runtime.taskSpeedLimitBytesPerSec",
    );
    push_if(
        &mut fields,
        patch.max_extract_bytes.is_some(),
        "runtime.maxExtractBytes",
    );
    push_if(
        &mut fields,
        patch.max_extract_files.is_some(),
        "runtime.maxExtractFiles",
    );
    push_if(
        &mut fields,
        patch.max_extract_depth.is_some(),
        "runtime.maxExtractDepth",
    );
    push_if(
        &mut fields,
        patch.index_enabled.is_some(),
        "runtime.indexEnabled",
    );
    push_if(
        &mut fields,
        patch.index_scan_delay_ms.is_some(),
        "runtime.indexScanDelayMs",
    );
    push_if(
        &mut fields,
        patch.audit_max_bytes.is_some(),
        "runtime.auditMaxBytes",
    );
    push_if(
        &mut fields,
        patch.audit_retention_files.is_some(),
        "runtime.auditRetentionFiles",
    );
    push_if(
        &mut fields,
        patch.trash_retention_days.is_some(),
        "runtime.trashRetentionDays",
    );
    push_if(
        &mut fields,
        patch.trash_max_bytes.is_some(),
        "runtime.trashMaxBytes",
    );
    push_if(
        &mut fields,
        patch.conflict_policy.is_some(),
        "runtime.conflictPolicy",
    );
    fields
}

fn push_if(fields: &mut Vec<&'static str>, touched: bool, field: &'static str) {
    if touched {
        fields.push(field);
    }
}

fn env_locked_fields() -> BTreeSet<String> {
    let mut fields = BTreeSet::new();
    for (field, env_name) in RUNTIME_ENV_FIELDS {
        if env::var_os(env_name).is_some() {
            fields.insert((*field).to_string());
        }
    }
    for (field, env_names) in STARTUP_ENV_FIELDS {
        if env_names.iter().any(|name| env::var_os(name).is_some()) {
            fields.insert((*field).to_string());
        }
    }
    fields
}

fn positive_u64(value: u64, name: &str) -> Result<u64, AppError> {
    if value == 0 {
        Err(AppError::bad_request(format!("{name}必须大于 0")))
    } else {
        Ok(value)
    }
}

fn positive_usize(value: usize, name: &str) -> Result<usize, AppError> {
    if value == 0 {
        Err(AppError::bad_request(format!("{name}必须大于 0")))
    } else {
        Ok(value)
    }
}

fn positive_optional_u64(value: Option<u64>, name: &str) -> Result<Option<u64>, AppError> {
    value.map(|value| positive_u64(value, name)).transpose()
}

fn positive_optional_usize(value: Option<usize>, name: &str) -> Result<Option<usize>, AppError> {
    value.map(|value| positive_usize(value, name)).transpose()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::models::ConflictPolicy;

    #[tokio::test]
    async fn patch_runtime_writes_config_and_updates_snapshot() {
        let root = temp_dir("settings-patch");
        let config_path = root.join("data/config.json");
        let mut config = AppConfig::load_from_file(config_path.clone()).unwrap();
        config.config_file = config_path.clone();
        let service = SettingsService::new(&config);

        let patch: RuntimeSettingsPatch = serde_json::from_value(json!({
            "maxUploadBytes": 8,
            "maxDirPageSize": 20,
            "editableExtensions": [".TXT", " md "],
            "conflictPolicy": "reject"
        }))
        .unwrap();

        let runtime = service.patch_runtime(patch).await.unwrap();

        assert_eq!(runtime.max_upload_bytes, Some(8));
        assert_eq!(runtime.max_dir_page_size, 20);
        assert_eq!(runtime.editable_extensions, vec!["txt", "md"]);
        assert_eq!(runtime.conflict_policy, ConflictPolicy::Reject);

        let text = fs::read_to_string(config_path).await.unwrap();
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["limits"]["maxUploadBytes"], 8);
        assert_eq!(value["limits"]["maxDirPageSize"], 20);
        assert_eq!(value["editor"]["editableExtensions"], json!(["txt", "md"]));
        assert_eq!(value["conflictPolicy"], "reject");

        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn reload_runtime_reads_config_file() {
        let root = temp_dir("settings-reload");
        let config_path = root.join("data/config.json");
        std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        std::fs::write(
            &config_path,
            r#"{"limits":{"maxDirPageSize":5},"conflictPolicy":"overwrite"}"#,
        )
        .unwrap();
        let config = AppConfig::load_from_file(config_path.clone()).unwrap();
        let service = SettingsService::new(&config);

        std::fs::write(
            &config_path,
            r#"{"limits":{"maxDirPageSize":9},"conflictPolicy":"reject"}"#,
        )
        .unwrap();

        let runtime = service.reload_runtime().await.unwrap();

        assert_eq!(runtime.max_dir_page_size, 9);
        assert_eq!(runtime.conflict_policy, ConflictPolicy::Reject);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_zero_positive_values() {
        let mut settings = AppConfig::load_from_file(PathBuf::from("missing.json"))
            .unwrap()
            .runtime_settings();
        let patch: RuntimeSettingsPatch = serde_json::from_value(json!({
            "maxDirPageSize": 0
        }))
        .unwrap();

        let error = apply_patch(&mut settings, patch).unwrap_err();

        assert!(matches!(error, AppError::BadRequest(_)));
    }

    fn temp_dir(prefix: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("web-file-browser-{prefix}-{nonce}"))
    }
}
