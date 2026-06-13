use std::{env, net::SocketAddr, path::PathBuf};

use crate::models::ConflictPolicy;

const DEFAULT_BIND_ADDRESS: &str = "0.0.0.0";
const DEFAULT_PORT: u16 = 8080;
const DEFAULT_MAPPING_FILE: &str = "data/mappings.json";
const DEFAULT_CONFIG_FILE: &str = "data/config.json";
const DEFAULT_TRASH_DIR: &str = "data/trash";
const DEFAULT_STATIC_DIR: &str = "ui/dist";
const DEFAULT_TRUST_PROXY_HEADERS: bool = false;
const DEFAULT_MAX_EDIT_BYTES: u64 = 2 * 1024 * 1024;
const DEFAULT_MAX_DIR_PAGE_SIZE: usize = 2_000;
const DEFAULT_AUDIT_FILE: &str = "data/audit.jsonl";
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
    pub initial_admin_password: Option<String>,
    pub trash_dir: PathBuf,
    pub static_dir: PathBuf,
    pub cors_allowed_origins: Vec<String>,
    pub trust_proxy_headers: bool,
    pub max_edit_bytes: u64,
    pub editable_extensions: Vec<String>,
    pub editable_mime_types: Vec<String>,
    pub max_upload_bytes: Option<u64>,
    pub max_dir_page_size: usize,
    pub audit_file: PathBuf,
    pub audit_max_bytes: Option<u64>,
    pub audit_retention_files: usize,
    pub max_dir_concurrency: usize,
    pub max_transfer_concurrency: usize,
    pub max_ip_concurrency: usize,
    pub max_task_concurrency: usize,
    pub task_history_limit: usize,
    pub task_speed_limit_bytes_per_sec: Option<u64>,
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
    pub fn from_env() -> Self {
        let bind_address =
            env::var("WEB_FILE_BROWSER_BIND").unwrap_or_else(|_| DEFAULT_BIND_ADDRESS.to_string());
        let port = env::var("PORT")
            .ok()
            .and_then(|port| port.parse().ok())
            .unwrap_or(DEFAULT_PORT);
        let mapping_file = env::var("WEB_FILE_BROWSER_MAPPING_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_MAPPING_FILE));
        let config_file = env::var("WEB_FILE_BROWSER_CONFIG_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_CONFIG_FILE));
        let initial_admin_password = env::var("WEB_FILE_BROWSER_ADMIN_PASSWORD")
            .ok()
            .filter(|password| !password.is_empty());
        let trash_dir = env::var("WEB_FILE_BROWSER_TRASH_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_TRASH_DIR));
        let static_dir = env::var("WEB_FILE_BROWSER_STATIC_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_STATIC_DIR));
        let cors_allowed_origins = env::var("WEB_FILE_BROWSER_CORS_ORIGINS")
            .ok()
            .map(|value| split_env_list(&value))
            .unwrap_or_default();
        let trust_proxy_headers = env_bool(
            "WEB_FILE_BROWSER_TRUST_PROXY_HEADERS",
            DEFAULT_TRUST_PROXY_HEADERS,
        );
        let max_edit_bytes = env::var("WEB_FILE_BROWSER_MAX_EDIT_BYTES")
            .ok()
            .and_then(|value| value.parse().ok())
            .filter(|value| *value > 0)
            .unwrap_or(DEFAULT_MAX_EDIT_BYTES);
        let editable_extensions = env::var("WEB_FILE_BROWSER_EDITABLE_EXTENSIONS")
            .ok()
            .map(|value| normalize_extension_list(&value))
            .unwrap_or_default();
        let editable_mime_types = env::var("WEB_FILE_BROWSER_EDITABLE_MIME_TYPES")
            .ok()
            .map(|value| normalize_mime_list(&value))
            .unwrap_or_default();
        let max_upload_bytes = env::var("WEB_FILE_BROWSER_MAX_UPLOAD_BYTES")
            .ok()
            .and_then(|value| value.parse().ok());
        let max_dir_page_size = env::var("WEB_FILE_BROWSER_MAX_DIR_PAGE_SIZE")
            .ok()
            .and_then(|value| value.parse().ok())
            .filter(|value| *value > 0)
            .unwrap_or(DEFAULT_MAX_DIR_PAGE_SIZE);
        let audit_file = env::var("WEB_FILE_BROWSER_AUDIT_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_AUDIT_FILE));
        let audit_max_bytes =
            env_optional_positive_u64("WEB_FILE_BROWSER_AUDIT_MAX_BYTES", DEFAULT_AUDIT_MAX_BYTES);
        let audit_retention_files = env_usize_allow_zero(
            "WEB_FILE_BROWSER_AUDIT_RETENTION_FILES",
            DEFAULT_AUDIT_RETENTION_FILES,
        );
        let max_dir_concurrency = env_usize(
            "WEB_FILE_BROWSER_MAX_DIR_CONCURRENCY",
            DEFAULT_MAX_DIR_CONCURRENCY,
        );
        let max_transfer_concurrency = env_usize(
            "WEB_FILE_BROWSER_MAX_TRANSFER_CONCURRENCY",
            DEFAULT_MAX_TRANSFER_CONCURRENCY,
        );
        let max_ip_concurrency = env_usize(
            "WEB_FILE_BROWSER_MAX_IP_CONCURRENCY",
            DEFAULT_MAX_IP_CONCURRENCY,
        );
        let max_task_concurrency = env_usize(
            "WEB_FILE_BROWSER_MAX_TASK_CONCURRENCY",
            DEFAULT_MAX_TASK_CONCURRENCY,
        );
        let task_history_limit = env_usize(
            "WEB_FILE_BROWSER_TASK_HISTORY_LIMIT",
            DEFAULT_TASK_HISTORY_LIMIT,
        );
        let task_speed_limit_bytes_per_sec =
            env::var("WEB_FILE_BROWSER_TASK_SPEED_LIMIT_BYTES_PER_SEC")
                .ok()
                .and_then(|value| value.parse().ok())
                .filter(|value| *value > 0);
        let max_extract_bytes = env::var("WEB_FILE_BROWSER_MAX_EXTRACT_BYTES")
            .ok()
            .and_then(|value| value.parse().ok())
            .filter(|value| *value > 0);
        let max_extract_files = env::var("WEB_FILE_BROWSER_MAX_EXTRACT_FILES")
            .ok()
            .and_then(|value| value.parse().ok())
            .filter(|value| *value > 0);
        let max_extract_depth = env_usize(
            "WEB_FILE_BROWSER_MAX_EXTRACT_DEPTH",
            DEFAULT_MAX_EXTRACT_DEPTH,
        );
        let index_enabled = env_bool("WEB_FILE_BROWSER_INDEX_ENABLED", DEFAULT_INDEX_ENABLED);
        let index_rebuild_on_startup = env_bool(
            "WEB_FILE_BROWSER_INDEX_REBUILD_ON_STARTUP",
            DEFAULT_INDEX_REBUILD_ON_STARTUP,
        );
        let index_scan_delay_ms = env::var("WEB_FILE_BROWSER_INDEX_SCAN_DELAY_MS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(DEFAULT_INDEX_SCAN_DELAY_MS);
        let trash_retention_days = env::var("WEB_FILE_BROWSER_TRASH_RETENTION_DAYS")
            .ok()
            .and_then(|value| value.parse().ok());
        let trash_max_bytes = env::var("WEB_FILE_BROWSER_TRASH_MAX_BYTES")
            .ok()
            .and_then(|value| value.parse().ok());
        let conflict_policy = env::var("WEB_FILE_BROWSER_CONFLICT_POLICY")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(DEFAULT_CONFLICT_POLICY);

        Self {
            bind_address,
            port,
            mapping_file,
            config_file,
            initial_admin_password,
            trash_dir,
            static_dir,
            cors_allowed_origins,
            trust_proxy_headers,
            max_edit_bytes,
            editable_extensions,
            editable_mime_types,
            max_upload_bytes,
            max_dir_page_size,
            audit_file,
            audit_max_bytes,
            audit_retention_files,
            max_dir_concurrency,
            max_transfer_concurrency,
            max_ip_concurrency,
            max_task_concurrency,
            task_history_limit,
            task_speed_limit_bytes_per_sec,
            max_extract_bytes,
            max_extract_files,
            max_extract_depth,
            index_enabled,
            index_rebuild_on_startup,
            index_scan_delay_ms,
            trash_retention_days,
            trash_max_bytes,
            conflict_policy,
        }
    }

    pub fn socket_addr(&self) -> Result<SocketAddr, std::net::AddrParseError> {
        format!("{}:{}", self.bind_address, self.port).parse()
    }
}

fn env_usize(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

fn env_usize_allow_zero(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn env_optional_positive_u64(name: &str, default: u64) -> Option<u64> {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .map(|value| if value == 0 { None } else { Some(value) })
        .unwrap_or(Some(default))
}

fn env_bool(name: &str, default: bool) -> bool {
    env::var(name)
        .ok()
        .map(|value| {
            matches!(
                value.as_str(),
                "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"
            )
        })
        .unwrap_or(default)
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
    split_env_list(value)
        .into_iter()
        .map(|value| value.trim_start_matches('.').trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect()
}

fn normalize_mime_list(value: &str) -> Vec<String> {
    split_env_list(value)
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
    use super::{normalize_extension_list, normalize_mime_list, split_env_list};

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
}
