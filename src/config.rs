use std::{
    env, fs,
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    models::{ConflictPolicy, RuntimeSettings, StartupSettings},
};

const DEFAULT_BIND_ADDRESS: &str = "0.0.0.0";
const DEFAULT_PORT: u16 = 8080;
const DEFAULT_MAPPING_FILE: &str = "data/mappings.json";
const DEFAULT_CONFIG_FILE: &str = "data/config.json";
const DEFAULT_AUTH_FILE: &str = "data/auth.json";
const DEFAULT_FAVORITES_FILE: &str = "data/favorites.json";
const DEFAULT_TRASH_DIR: &str = "data/trash";
const DEFAULT_STATIC_DIR: &str = "ui/dist";
const DEFAULT_TRUST_PROXY_HEADERS: bool = false;
const DEFAULT_AUTH_SESSION_TTL_SECONDS: u64 = 7 * 24 * 60 * 60;
const DEFAULT_AUTH_SECURE_COOKIE: bool = false;
const DEFAULT_MAX_EDIT_BYTES: u64 = 2 * 1024 * 1024;
const DEFAULT_MAX_DIR_PAGE_SIZE: usize = 2_000;
const DEFAULT_AUDIT_FILE: &str = "data/audit.jsonl";
const DEFAULT_AUDIT_ENABLED: bool = true;
const DEFAULT_AUDIT_MAX_BYTES: u64 = 10 * 1024 * 1024;
const DEFAULT_AUDIT_RETENTION_FILES: usize = 8;
const DEFAULT_MAX_DIR_CONCURRENCY: usize = 4;
const DEFAULT_MAX_TRANSFER_CONCURRENCY: usize = 8;
const DEFAULT_MAX_IP_CONCURRENCY: usize = 16;
const DEFAULT_MAX_TASK_CONCURRENCY: usize = 2;
const DEFAULT_TASK_HISTORY_LIMIT: usize = 200;
const DEFAULT_MAX_EXTRACT_DEPTH: usize = 64;
const DEFAULT_INDEX_ENABLED: bool = false;
const DEFAULT_INDEX_REBUILD_ON_STARTUP: bool = false;
const DEFAULT_INDEX_SCAN_DELAY_MS: u64 = 2;
const DEFAULT_CONFLICT_POLICY: ConflictPolicy = ConflictPolicy::AutoRename;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub bind_address: String,
    pub port: u16,
    pub mapping_file: PathBuf,
    pub config_file: PathBuf,
    pub auth_file: PathBuf,
    pub favorites_file: PathBuf,
    pub trash_dir: PathBuf,
    pub static_dir: PathBuf,
    pub cors_allowed_origins: Vec<String>,
    pub trust_proxy_headers: bool,
    pub auth_session_ttl_seconds: u64,
    pub auth_secure_cookie: bool,
    pub max_edit_bytes: u64,
    pub editable_extensions: Vec<String>,
    pub editable_mime_types: Vec<String>,
    pub max_upload_bytes: Option<u64>,
    pub max_dir_page_size: usize,
    pub audit_file: PathBuf,
    pub audit_enabled: bool,
    pub audit_max_bytes: Option<u64>,
    pub audit_retention_files: usize,
    pub max_dir_concurrency: usize,
    pub max_transfer_concurrency: usize,
    pub max_ip_concurrency: usize,
    pub max_task_concurrency: usize,
    pub task_history_limit: usize,
    pub task_speed_limit_bytes_per_sec: Option<u64>,
    pub max_archive_bytes: Option<u64>,
    pub max_archive_files: Option<usize>,
    pub max_extract_bytes: Option<u64>,
    pub max_extract_files: Option<usize>,
    pub max_extract_depth: usize,
    pub index_enabled: bool,
    pub index_rebuild_on_startup: bool,
    pub index_scan_delay_ms: u64,
    pub trash_retention_days: Option<u64>,
    pub trash_max_bytes: Option<u64>,
    pub conflict_policy: ConflictPolicy,
}

impl AppConfig {
    pub fn load() -> Result<Self, AppError> {
        let config_file = env_config_file();
        Self::load_from_file(config_file)
    }

    pub(crate) fn load_from_file(config_file: PathBuf) -> Result<Self, AppError> {
        let file_config = RuntimeConfigFile::read(&config_file)?;
        let mut config = Self::defaults(config_file);
        config.apply_file(file_config);
        config.apply_env();
        Ok(config)
    }

    pub fn socket_addr(&self) -> Result<SocketAddr, std::net::AddrParseError> {
        match self.bind_address.parse::<IpAddr>() {
            Ok(ip) => Ok(SocketAddr::new(ip, self.port)),
            Err(_) => format!("{}:{}", self.bind_address, self.port).parse(),
        }
    }

    pub(crate) fn runtime_settings(&self) -> RuntimeSettings {
        RuntimeSettings {
            auth_session_ttl_seconds: self.auth_session_ttl_seconds,
            auth_secure_cookie: self.auth_secure_cookie,
            max_edit_bytes: self.max_edit_bytes,
            editable_extensions: self.editable_extensions.clone(),
            editable_mime_types: self.editable_mime_types.clone(),
            max_upload_bytes: self.max_upload_bytes,
            max_dir_page_size: self.max_dir_page_size,
            max_dir_concurrency: self.max_dir_concurrency,
            max_transfer_concurrency: self.max_transfer_concurrency,
            max_ip_concurrency: self.max_ip_concurrency,
            max_task_concurrency: self.max_task_concurrency,
            task_history_limit: self.task_history_limit,
            task_speed_limit_bytes_per_sec: self.task_speed_limit_bytes_per_sec,
            max_archive_bytes: self.max_archive_bytes,
            max_archive_files: self.max_archive_files,
            max_extract_bytes: self.max_extract_bytes,
            max_extract_files: self.max_extract_files,
            max_extract_depth: self.max_extract_depth,
            index_enabled: self.index_enabled,
            index_scan_delay_ms: self.index_scan_delay_ms,
            audit_enabled: self.audit_enabled,
            audit_max_bytes: self.audit_max_bytes,
            audit_retention_files: self.audit_retention_files,
            trash_retention_days: self.trash_retention_days,
            trash_max_bytes: self.trash_max_bytes,
            conflict_policy: self.conflict_policy,
        }
    }

    pub(crate) fn startup_settings(&self) -> StartupSettings {
        StartupSettings {
            bind_address: self.bind_address.clone(),
            port: self.port,
            mapping_file: path_to_string(&self.mapping_file),
            config_file: path_to_string(&self.config_file),
            auth_file: path_to_string(&self.auth_file),
            favorites_file: path_to_string(&self.favorites_file),
            trash_dir: path_to_string(&self.trash_dir),
            static_dir: path_to_string(&self.static_dir),
            cors_allowed_origins: self.cors_allowed_origins.clone(),
            trust_proxy_headers: self.trust_proxy_headers,
            audit_file: path_to_string(&self.audit_file),
            index_rebuild_on_startup: self.index_rebuild_on_startup,
        }
    }

    fn defaults(config_file: PathBuf) -> Self {
        Self {
            bind_address: DEFAULT_BIND_ADDRESS.to_string(),
            port: DEFAULT_PORT,
            mapping_file: PathBuf::from(DEFAULT_MAPPING_FILE),
            config_file,
            auth_file: PathBuf::from(DEFAULT_AUTH_FILE),
            favorites_file: PathBuf::from(DEFAULT_FAVORITES_FILE),
            trash_dir: PathBuf::from(DEFAULT_TRASH_DIR),
            static_dir: PathBuf::from(DEFAULT_STATIC_DIR),
            cors_allowed_origins: Vec::new(),
            trust_proxy_headers: DEFAULT_TRUST_PROXY_HEADERS,
            auth_session_ttl_seconds: DEFAULT_AUTH_SESSION_TTL_SECONDS,
            auth_secure_cookie: DEFAULT_AUTH_SECURE_COOKIE,
            max_edit_bytes: DEFAULT_MAX_EDIT_BYTES,
            editable_extensions: Vec::new(),
            editable_mime_types: Vec::new(),
            max_upload_bytes: None,
            max_dir_page_size: DEFAULT_MAX_DIR_PAGE_SIZE,
            audit_file: PathBuf::from(DEFAULT_AUDIT_FILE),
            audit_enabled: DEFAULT_AUDIT_ENABLED,
            audit_max_bytes: Some(DEFAULT_AUDIT_MAX_BYTES),
            audit_retention_files: DEFAULT_AUDIT_RETENTION_FILES,
            max_dir_concurrency: DEFAULT_MAX_DIR_CONCURRENCY,
            max_transfer_concurrency: DEFAULT_MAX_TRANSFER_CONCURRENCY,
            max_ip_concurrency: DEFAULT_MAX_IP_CONCURRENCY,
            max_task_concurrency: DEFAULT_MAX_TASK_CONCURRENCY,
            task_history_limit: DEFAULT_TASK_HISTORY_LIMIT,
            task_speed_limit_bytes_per_sec: None,
            max_archive_bytes: None,
            max_archive_files: None,
            max_extract_bytes: None,
            max_extract_files: None,
            max_extract_depth: DEFAULT_MAX_EXTRACT_DEPTH,
            index_enabled: DEFAULT_INDEX_ENABLED,
            index_rebuild_on_startup: DEFAULT_INDEX_REBUILD_ON_STARTUP,
            index_scan_delay_ms: DEFAULT_INDEX_SCAN_DELAY_MS,
            trash_retention_days: None,
            trash_max_bytes: None,
            conflict_policy: DEFAULT_CONFLICT_POLICY,
        }
    }

    fn apply_file(&mut self, file_config: RuntimeConfigFile) {
        if let Some(server) = file_config.server {
            assign(&mut self.bind_address, server.bind);
            assign(&mut self.port, server.port);
            assign(&mut self.static_dir, server.static_dir);
            assign(&mut self.cors_allowed_origins, server.cors_allowed_origins);
            assign(&mut self.trust_proxy_headers, server.trust_proxy_headers);
        }

        if let Some(storage) = file_config.storage {
            assign(&mut self.mapping_file, storage.mapping_file);
            assign(&mut self.auth_file, storage.auth_file);
            assign(&mut self.favorites_file, storage.favorites_file);
            assign(&mut self.trash_dir, storage.trash_dir);
            assign(&mut self.audit_file, storage.audit_file);
        }

        if let Some(auth) = file_config.auth {
            assign_positive(&mut self.auth_session_ttl_seconds, auth.session_ttl_seconds);
            assign(&mut self.auth_secure_cookie, auth.secure_cookie);
        }

        if let Some(limits) = file_config.limits {
            assign(&mut self.max_upload_bytes, limits.max_upload_bytes);
            assign_positive(&mut self.max_dir_page_size, limits.max_dir_page_size);
            assign_positive(&mut self.max_dir_concurrency, limits.max_dir_concurrency);
            assign_positive(
                &mut self.max_transfer_concurrency,
                limits.max_transfer_concurrency,
            );
            assign_positive(&mut self.max_ip_concurrency, limits.max_ip_concurrency);
        }

        if let Some(editor) = file_config.editor {
            assign_positive(&mut self.max_edit_bytes, editor.max_edit_bytes);
            assign(
                &mut self.editable_extensions,
                editor.editable_extensions.map(normalize_extension_values),
            );
            assign(
                &mut self.editable_mime_types,
                editor.editable_mime_types.map(normalize_mime_values),
            );
        }

        if let Some(tasks) = file_config.tasks {
            assign_positive(&mut self.max_task_concurrency, tasks.max_concurrency);
            assign_positive(&mut self.task_history_limit, tasks.history_limit);
            assign_positive_option(
                &mut self.task_speed_limit_bytes_per_sec,
                tasks.speed_limit_bytes_per_sec,
            );
        }

        if let Some(archive) = file_config.archive {
            assign_positive_option(&mut self.max_archive_bytes, archive.max_archive_bytes);
            assign_positive_option(&mut self.max_archive_files, archive.max_archive_files);
            assign_positive_option(&mut self.max_extract_bytes, archive.max_extract_bytes);
            assign_positive_option(&mut self.max_extract_files, archive.max_extract_files);
            assign_positive(&mut self.max_extract_depth, archive.max_extract_depth);
        }

        if let Some(index) = file_config.index {
            assign(&mut self.index_enabled, index.enabled);
            assign(&mut self.index_rebuild_on_startup, index.rebuild_on_startup);
            assign(&mut self.index_scan_delay_ms, index.scan_delay_ms);
        }

        if let Some(trash) = file_config.trash {
            assign_positive_option(&mut self.trash_retention_days, trash.retention_days);
            assign_positive_option(&mut self.trash_max_bytes, trash.max_bytes);
        }

        if let Some(audit) = file_config.audit {
            assign(&mut self.audit_enabled, audit.enabled);
            assign_zero_disables(&mut self.audit_max_bytes, audit.max_bytes);
            assign(&mut self.audit_retention_files, audit.retention_files);
        }

        assign(&mut self.conflict_policy, file_config.conflict_policy);
    }

    fn apply_env(&mut self) {
        assign_env_string(&mut self.bind_address, "WEB_FILE_BROWSER_BIND");
        assign_env_u16(&mut self.port, "PORT");
        assign_env_path(&mut self.mapping_file, "WEB_FILE_BROWSER_MAPPING_FILE");
        assign_env_path(&mut self.auth_file, "WEB_FILE_BROWSER_AUTH_FILE");
        assign_env_path(&mut self.favorites_file, "WEB_FILE_BROWSER_FAVORITES_FILE");
        assign_env_path(&mut self.trash_dir, "WEB_FILE_BROWSER_TRASH_DIR");
        assign_env_path(&mut self.static_dir, "WEB_FILE_BROWSER_STATIC_DIR");
        assign_env_list(
            &mut self.cors_allowed_origins,
            "WEB_FILE_BROWSER_CORS_ORIGINS",
            split_env_list,
        );
        assign_env_bool(
            &mut self.trust_proxy_headers,
            "WEB_FILE_BROWSER_TRUST_PROXY_HEADERS",
        );
        assign_env_u64_positive(
            &mut self.auth_session_ttl_seconds,
            "WEB_FILE_BROWSER_AUTH_SESSION_TTL_SECONDS",
        );
        assign_env_bool(
            &mut self.auth_secure_cookie,
            "WEB_FILE_BROWSER_AUTH_SECURE_COOKIE",
        );
        assign_env_u64_positive(&mut self.max_edit_bytes, "WEB_FILE_BROWSER_MAX_EDIT_BYTES");
        assign_env_list(
            &mut self.editable_extensions,
            "WEB_FILE_BROWSER_EDITABLE_EXTENSIONS",
            normalize_extension_list,
        );
        assign_env_list(
            &mut self.editable_mime_types,
            "WEB_FILE_BROWSER_EDITABLE_MIME_TYPES",
            normalize_mime_list,
        );
        assign_env_option_u64(
            &mut self.max_upload_bytes,
            "WEB_FILE_BROWSER_MAX_UPLOAD_BYTES",
        );
        assign_env_usize_positive(
            &mut self.max_dir_page_size,
            "WEB_FILE_BROWSER_MAX_DIR_PAGE_SIZE",
        );
        assign_env_path(&mut self.audit_file, "WEB_FILE_BROWSER_AUDIT_FILE");
        assign_env_bool(&mut self.audit_enabled, "WEB_FILE_BROWSER_AUDIT_ENABLED");
        assign_env_zero_disables_u64(
            &mut self.audit_max_bytes,
            "WEB_FILE_BROWSER_AUDIT_MAX_BYTES",
        );
        assign_env_usize(
            &mut self.audit_retention_files,
            "WEB_FILE_BROWSER_AUDIT_RETENTION_FILES",
        );
        assign_env_usize_positive(
            &mut self.max_dir_concurrency,
            "WEB_FILE_BROWSER_MAX_DIR_CONCURRENCY",
        );
        assign_env_usize_positive(
            &mut self.max_transfer_concurrency,
            "WEB_FILE_BROWSER_MAX_TRANSFER_CONCURRENCY",
        );
        assign_env_usize_positive(
            &mut self.max_ip_concurrency,
            "WEB_FILE_BROWSER_MAX_IP_CONCURRENCY",
        );
        assign_env_usize_positive(
            &mut self.max_task_concurrency,
            "WEB_FILE_BROWSER_MAX_TASK_CONCURRENCY",
        );
        assign_env_usize_positive(
            &mut self.task_history_limit,
            "WEB_FILE_BROWSER_TASK_HISTORY_LIMIT",
        );
        assign_env_option_u64(
            &mut self.task_speed_limit_bytes_per_sec,
            "WEB_FILE_BROWSER_TASK_SPEED_LIMIT_BYTES_PER_SEC",
        );
        assign_env_option_u64(
            &mut self.max_archive_bytes,
            "WEB_FILE_BROWSER_MAX_ARCHIVE_BYTES",
        );
        assign_env_option_usize(
            &mut self.max_archive_files,
            "WEB_FILE_BROWSER_MAX_ARCHIVE_FILES",
        );
        assign_env_option_u64(
            &mut self.max_extract_bytes,
            "WEB_FILE_BROWSER_MAX_EXTRACT_BYTES",
        );
        assign_env_option_usize(
            &mut self.max_extract_files,
            "WEB_FILE_BROWSER_MAX_EXTRACT_FILES",
        );
        assign_env_usize_positive(
            &mut self.max_extract_depth,
            "WEB_FILE_BROWSER_MAX_EXTRACT_DEPTH",
        );
        assign_env_bool(&mut self.index_enabled, "WEB_FILE_BROWSER_INDEX_ENABLED");
        assign_env_bool(
            &mut self.index_rebuild_on_startup,
            "WEB_FILE_BROWSER_INDEX_REBUILD_ON_STARTUP",
        );
        assign_env_u64(
            &mut self.index_scan_delay_ms,
            "WEB_FILE_BROWSER_INDEX_SCAN_DELAY_MS",
        );
        assign_env_option_u64(
            &mut self.trash_retention_days,
            "WEB_FILE_BROWSER_TRASH_RETENTION_DAYS",
        );
        assign_env_option_u64(
            &mut self.trash_max_bytes,
            "WEB_FILE_BROWSER_TRASH_MAX_BYTES",
        );
        assign_env_conflict_policy(
            &mut self.conflict_policy,
            "WEB_FILE_BROWSER_CONFLICT_POLICY",
        );
    }
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RuntimeConfigFile {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) server: Option<ServerConfigFile>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) storage: Option<StorageConfigFile>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) auth: Option<AuthConfigFile>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) limits: Option<LimitsConfigFile>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) editor: Option<EditorConfigFile>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tasks: Option<TaskConfigFile>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) archive: Option<ArchiveConfigFile>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) index: Option<IndexConfigFile>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) trash: Option<TrashConfigFile>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) audit: Option<AuditConfigFile>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) conflict_policy: Option<ConflictPolicy>,
}

impl RuntimeConfigFile {
    pub(crate) fn read(path: &PathBuf) -> Result<Self, AppError> {
        let text = match fs::read_to_string(path) {
            Ok(text) if text.trim().is_empty() => return Ok(Self::default()),
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::default());
            }
            Err(error) => return Err(error.into()),
        };

        serde_json::from_str(&text).map_err(|error| {
            AppError::internal(format!(
                "读取运行配置文件失败: {}，请检查 JSON 格式: {error}",
                path.display()
            ))
        })
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ServerConfigFile {
    #[serde(default, alias = "bindAddress")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) bind: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) port: Option<u16>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) static_dir: Option<PathBuf>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) cors_allowed_origins: Option<Vec<String>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) trust_proxy_headers: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageConfigFile {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) mapping_file: Option<PathBuf>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) auth_file: Option<PathBuf>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) favorites_file: Option<PathBuf>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) trash_dir: Option<PathBuf>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) audit_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AuthConfigFile {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) session_ttl_seconds: Option<u64>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) secure_cookie: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LimitsConfigFile {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) max_upload_bytes: Option<Option<u64>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) max_dir_page_size: Option<usize>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) max_dir_concurrency: Option<usize>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) max_transfer_concurrency: Option<usize>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) max_ip_concurrency: Option<usize>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EditorConfigFile {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) max_edit_bytes: Option<u64>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) editable_extensions: Option<Vec<String>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) editable_mime_types: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TaskConfigFile {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) max_concurrency: Option<usize>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) history_limit: Option<usize>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) speed_limit_bytes_per_sec: Option<Option<u64>>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ArchiveConfigFile {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) max_archive_bytes: Option<Option<u64>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) max_archive_files: Option<Option<usize>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) max_extract_bytes: Option<Option<u64>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) max_extract_files: Option<Option<usize>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) max_extract_depth: Option<usize>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IndexConfigFile {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) enabled: Option<bool>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) rebuild_on_startup: Option<bool>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) scan_delay_ms: Option<u64>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TrashConfigFile {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) retention_days: Option<Option<u64>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) max_bytes: Option<Option<u64>>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AuditConfigFile {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) enabled: Option<bool>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) max_bytes: Option<Option<u64>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) retention_files: Option<usize>,
}

fn env_config_file() -> PathBuf {
    env::var("WEB_FILE_BROWSER_CONFIG_FILE")
        .or_else(|_| env::var("WEB_FILE_BROWSER_CONFIG"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_CONFIG_FILE))
}

fn assign<T>(target: &mut T, value: Option<T>) {
    if let Some(value) = value {
        *target = value;
    }
}

fn assign_positive<T>(target: &mut T, value: Option<T>)
where
    T: PartialOrd + From<u8>,
{
    if let Some(value) = value.filter(|value| *value > T::from(0)) {
        *target = value;
    }
}

fn assign_positive_option<T>(target: &mut Option<T>, value: Option<Option<T>>)
where
    T: PartialOrd + From<u8>,
{
    if let Some(value) = value {
        *target = value.filter(|value| *value > T::from(0));
    }
}

fn assign_zero_disables(target: &mut Option<u64>, value: Option<Option<u64>>) {
    if let Some(value) = value {
        *target = value.and_then(|value| if value == 0 { None } else { Some(value) });
    }
}

fn assign_env_string(target: &mut String, name: &str) {
    if let Ok(value) = env::var(name)
        && !value.is_empty()
    {
        *target = value;
    }
}

fn assign_env_path(target: &mut PathBuf, name: &str) {
    if let Ok(value) = env::var(name)
        && !value.is_empty()
    {
        *target = PathBuf::from(value);
    }
}

fn assign_env_u16(target: &mut u16, name: &str) {
    if let Ok(value) = env::var(name).unwrap_or_default().parse()
        && value > 0
    {
        *target = value;
    }
}

fn assign_env_u64(target: &mut u64, name: &str) {
    if let Ok(value) = env::var(name).unwrap_or_default().parse() {
        *target = value;
    }
}

fn assign_env_u64_positive(target: &mut u64, name: &str) {
    if let Ok(value) = env::var(name).unwrap_or_default().parse()
        && value > 0
    {
        *target = value;
    }
}

fn assign_env_usize(target: &mut usize, name: &str) {
    if let Ok(value) = env::var(name).unwrap_or_default().parse() {
        *target = value;
    }
}

fn assign_env_usize_positive(target: &mut usize, name: &str) {
    if let Ok(value) = env::var(name).unwrap_or_default().parse()
        && value > 0
    {
        *target = value;
    }
}

fn assign_env_option_u64(target: &mut Option<u64>, name: &str) {
    if let Ok(value) = env::var(name) {
        *target = value.parse().ok().filter(|value| *value > 0);
    }
}

fn assign_env_option_usize(target: &mut Option<usize>, name: &str) {
    if let Ok(value) = env::var(name) {
        *target = value.parse().ok().filter(|value| *value > 0);
    }
}

fn assign_env_zero_disables_u64(target: &mut Option<u64>, name: &str) {
    if let Ok(value) = env::var(name)
        && let Ok(value) = value.parse()
    {
        *target = if value == 0 { None } else { Some(value) };
    }
}

fn assign_env_bool(target: &mut bool, name: &str) {
    if let Ok(value) = env::var(name) {
        *target = matches!(
            value.as_str(),
            "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"
        );
    }
}

fn assign_env_list(
    target: &mut Vec<String>,
    name: &str,
    normalize: impl FnOnce(&str) -> Vec<String>,
) {
    if let Ok(value) = env::var(name) {
        *target = normalize(&value);
    }
}

fn assign_env_conflict_policy(target: &mut ConflictPolicy, name: &str) {
    if let Ok(value) = env::var(name)
        && let Ok(value) = value.parse()
    {
        *target = value;
    }
}

fn split_env_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn normalize_extension_list(value: &str) -> Vec<String> {
    normalize_extension_values(split_env_list(value))
}

pub(crate) fn normalize_extension_values(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .map(|value| value.trim_start_matches('.').trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect()
}

fn normalize_mime_list(value: &str) -> Vec<String> {
    normalize_mime_values(split_env_list(value))
}

pub(crate) fn normalize_mime_values(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .map(|value| value.to_ascii_lowercase())
        .filter(|value| {
            value == "*/*"
                || value
                    .split_once('/')
                    .is_some_and(|(group, subtype)| !group.is_empty() && !subtype.is_empty())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        AppConfig, RuntimeConfigFile, normalize_extension_list, normalize_mime_list, split_env_list,
    };

    #[test]
    fn splits_comma_separated_env_list() {
        assert_eq!(
            split_env_list(" http://localhost:5173, http://192.168.1.10:5173 ,, "),
            vec![
                "http://localhost:5173".to_string(),
                "http://192.168.1.10:5173".to_string(),
            ]
        );
    }

    #[test]
    fn normalizes_editable_extension_list() {
        assert_eq!(
            normalize_extension_list(" .TXT, rs, , .Vue "),
            vec!["txt".to_string(), "rs".to_string(), "vue".to_string()]
        );
    }

    #[test]
    fn normalizes_editable_mime_list() {
        assert_eq!(
            normalize_mime_list(" Text/*, application/json, broken, image/ "),
            vec!["text/*".to_string(), "application/json".to_string()]
        );
    }

    #[test]
    fn runtime_config_file_accepts_nested_config() {
        let config: RuntimeConfigFile = serde_json::from_str(
            r#"{
                "server": { "bind": "127.0.0.1", "port": 18080 },
                "storage": { "authFile": "data/auth.json" },
                "editor": { "editableExtensions": [".TXT", "md"] },
                "conflictPolicy": "reject"
            }"#,
        )
        .unwrap();

        assert_eq!(config.server.unwrap().port, Some(18080));
        assert_eq!(
            config.editor.unwrap().editable_extensions.unwrap(),
            vec![".TXT".to_string(), "md".to_string()]
        );
        assert_eq!(
            config.conflict_policy,
            Some(crate::models::ConflictPolicy::Reject)
        );
    }

    #[test]
    fn socket_addr_supports_ipv6_bind_address() {
        let mut config = AppConfig::load_from_file("missing.json".into()).unwrap();
        config.bind_address = "::".to_string();
        config.port = 18080;

        let address = config.socket_addr().unwrap();

        assert_eq!(address.to_string(), "[::]:18080");
    }
}
