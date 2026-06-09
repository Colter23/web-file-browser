use std::{env, net::SocketAddr, path::PathBuf};

const DEFAULT_BIND_ADDRESS: &str = "0.0.0.0";
const DEFAULT_PORT: u16 = 8080;
const DEFAULT_MAPPING_FILE: &str = "data/mappings.json";
const DEFAULT_CONFIG_FILE: &str = "data/config.json";
const DEFAULT_TRASH_DIR: &str = "data/trash";
const DEFAULT_STATIC_DIR: &str = "ui/dist";
const DEFAULT_MAX_DIR_PAGE_SIZE: usize = 2_000;
const DEFAULT_AUDIT_FILE: &str = "data/audit.jsonl";
const DEFAULT_MAX_DIR_CONCURRENCY: usize = 4;
const DEFAULT_MAX_TRANSFER_CONCURRENCY: usize = 8;
const DEFAULT_MAX_IP_CONCURRENCY: usize = 16;
const DEFAULT_MAX_TASK_CONCURRENCY: usize = 2;
const DEFAULT_INDEX_ENABLED: bool = false;
const DEFAULT_INDEX_SCAN_DELAY_MS: u64 = 2;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub bind_address: String,
    pub port: u16,
    pub mapping_file: PathBuf,
    pub config_file: PathBuf,
    pub trash_dir: PathBuf,
    pub static_dir: PathBuf,
    pub max_upload_bytes: Option<u64>,
    pub max_dir_page_size: usize,
    pub audit_file: PathBuf,
    pub max_dir_concurrency: usize,
    pub max_transfer_concurrency: usize,
    pub max_ip_concurrency: usize,
    pub max_task_concurrency: usize,
    pub task_speed_limit_bytes_per_sec: Option<u64>,
    pub index_enabled: bool,
    pub index_scan_delay_ms: u64,
    pub trash_retention_days: Option<u64>,
    pub trash_max_bytes: Option<u64>,
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
        let trash_dir = env::var("WEB_FILE_BROWSER_TRASH_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_TRASH_DIR));
        let static_dir = env::var("WEB_FILE_BROWSER_STATIC_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_STATIC_DIR));
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
        let task_speed_limit_bytes_per_sec =
            env::var("WEB_FILE_BROWSER_TASK_SPEED_LIMIT_BYTES_PER_SEC")
                .ok()
                .and_then(|value| value.parse().ok())
                .filter(|value| *value > 0);
        let index_enabled = env_bool("WEB_FILE_BROWSER_INDEX_ENABLED", DEFAULT_INDEX_ENABLED);
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

        Self {
            bind_address,
            port,
            mapping_file,
            config_file,
            trash_dir,
            static_dir,
            max_upload_bytes,
            max_dir_page_size,
            audit_file,
            max_dir_concurrency,
            max_transfer_concurrency,
            max_ip_concurrency,
            max_task_concurrency,
            task_speed_limit_bytes_per_sec,
            index_enabled,
            index_scan_delay_ms,
            trash_retention_days,
            trash_max_bytes,
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
