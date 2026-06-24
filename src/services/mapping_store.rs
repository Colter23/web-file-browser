use std::{path::PathBuf, sync::Arc};
use tokio::sync::RwLock;

use crate::{
    error::AppError,
    models::{FolderNode, PathMapping, ReorderMappingsRequest},
    services::{
        path_display::{display_path, has_windows_extended_prefix},
        path_resolver::MappingSnapshot,
    },
};

#[derive(Clone)]
pub struct MappingStore {
    file_path: Arc<PathBuf>,
    mappings: Arc<RwLock<Vec<PathMapping>>>,
    snapshot: Arc<RwLock<Arc<MappingSnapshot>>>,
}

impl MappingStore {
    pub async fn load(file_path: PathBuf) -> Result<Self, AppError> {
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let loaded_mappings = match tokio::fs::read_to_string(&file_path).await {
            Ok(text) if text.trim().is_empty() => Vec::new(),
            Ok(text) => serde_json::from_str(&text)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(error) => return Err(error.into()),
        };

        let snapshot = MappingSnapshot::build(sorted_mappings(&loaded_mappings)).await?;
        let mappings = snapshot.mappings.clone();
        if loaded_mappings
            .iter()
            .any(|mapping| has_windows_extended_prefix(&mapping.folder_path))
        {
            let text = serde_json::to_vec_pretty(&mappings)?;
            tokio::fs::write(&file_path, text).await?;
        }

        Ok(Self {
            file_path: Arc::new(file_path),
            mappings: Arc::new(RwLock::new(mappings)),
            snapshot: Arc::new(RwLock::new(snapshot)),
        })
    }

    pub async fn list(&self) -> Vec<PathMapping> {
        self.snapshot.read().await.mappings.clone()
    }

    pub async fn count(&self) -> usize {
        self.snapshot.read().await.mappings.len()
    }

    pub async fn get(&self, id: i64) -> Option<PathMapping> {
        self.snapshot
            .read()
            .await
            .mappings
            .iter()
            .find(|mapping| mapping.id == Some(id))
            .cloned()
    }

    pub async fn snapshot(&self) -> Arc<MappingSnapshot> {
        self.snapshot.read().await.clone()
    }

    pub async fn root_node(&self) -> Option<FolderNode> {
        self.snapshot.read().await.root_node()
    }

    pub async fn create(&self, mapping: PathMapping) -> Result<i64, AppError> {
        let mut mapping = normalize_mapping(mapping).await?;
        let mut mappings = self.mappings.write().await;

        validate_mount_conflict(&mappings, &mapping.mount_path, None)?;
        let id = next_id(&mappings);
        mapping.id = Some(id);

        mappings.push(mapping);
        let next = mappings.clone();
        drop(mappings);
        self.save_and_refresh(&next).await?;

        Ok(id)
    }

    pub async fn update(&self, id: i64, mapping: PathMapping) -> Result<(), AppError> {
        let mut mapping = normalize_mapping(mapping).await?;
        mapping.id = Some(id);

        let mut mappings = self.mappings.write().await;
        let Some(index) = mappings.iter().position(|current| current.id == Some(id)) else {
            return Err(AppError::not_found(format!("查无此映射: {id}")));
        };

        validate_mount_conflict(&mappings, &mapping.mount_path, Some(id))?;
        mappings[index] = mapping;
        let next = mappings.clone();
        drop(mappings);
        self.save_and_refresh(&next).await
    }

    pub async fn reorder(&self, request: ReorderMappingsRequest) -> Result<(), AppError> {
        let mut mappings = self.mappings.write().await;
        for requested in &request.items {
            if !mappings
                .iter()
                .any(|mapping| mapping.id == Some(requested.id))
            {
                return Err(AppError::not_found(format!("查无此映射: {}", requested.id)));
            }
        }

        for requested in request.items {
            if let Some(mapping) = mappings
                .iter_mut()
                .find(|mapping| mapping.id == Some(requested.id))
            {
                mapping.order = Some(requested.order);
            }
        }

        let next = sorted_mappings(&mappings);
        *mappings = next.clone();
        drop(mappings);
        self.save_and_refresh(&next).await
    }

    pub async fn delete(&self, id: i64) -> Result<(), AppError> {
        let mut mappings = self.mappings.write().await;
        let before_len = mappings.len();
        mappings.retain(|mapping| mapping.id != Some(id));

        if mappings.len() == before_len {
            return Err(AppError::not_found(format!("查无此映射: {id}")));
        }

        let next = mappings.clone();
        drop(mappings);
        self.save_and_refresh(&next).await
    }

    async fn save_and_refresh(&self, mappings: &[PathMapping]) -> Result<(), AppError> {
        if let Some(parent) = self.file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let text = serde_json::to_vec_pretty(mappings)?;
        tokio::fs::write(&*self.file_path, text).await?;
        let snapshot = MappingSnapshot::build(sorted_mappings(mappings)).await?;
        *self.snapshot.write().await = snapshot;
        Ok(())
    }
}

async fn normalize_mapping(mut mapping: PathMapping) -> Result<PathMapping, AppError> {
    mapping.mount_path = normalize_mount_path(&mapping.mount_path)?;

    let raw_folder_path = mapping.folder_path.trim();
    if raw_folder_path.is_empty() {
        return Err(AppError::bad_request("本地文件路径不能为空"));
    }

    let folder_path = tokio::fs::canonicalize(raw_folder_path)
        .await
        .map_err(|_| AppError::bad_request(format!("查无此路径: {raw_folder_path}")))?;
    if !folder_path.is_dir() {
        return Err(AppError::bad_request(format!(
            "路径不是文件夹: {raw_folder_path}"
        )));
    }

    mapping.folder_path = display_path(&folder_path);
    mapping.remark = Some(mapping.remark.unwrap_or_default());
    mapping.order = Some(mapping.order.unwrap_or(0));
    Ok(mapping)
}

pub fn normalize_mount_path(raw: &str) -> Result<String, AppError> {
    let raw = raw.trim().replace('\\', "/");
    if raw.is_empty() {
        return Err(AppError::bad_request("挂载路径不能为空"));
    }

    let raw = if raw.starts_with('/') {
        raw
    } else {
        format!("/{raw}")
    };

    let mut parts = Vec::new();
    for part in raw.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            return Err(AppError::bad_request("挂载路径不能包含 .."));
        }
        parts.push(part);
    }

    if parts.is_empty() {
        Ok("/".to_string())
    } else {
        Ok(format!("/{}", parts.join("/")))
    }
}

fn validate_mount_conflict(
    mappings: &[PathMapping],
    new_mount_path: &str,
    ignore_id: Option<i64>,
) -> Result<(), AppError> {
    if new_mount_path == "/" && !mappings.is_empty() {
        let only_ignored = mappings.iter().all(|mapping| mapping.id == ignore_id);
        if !only_ignored {
            return Err(AppError::conflict("路径映射冲突"));
        }
    }

    for mapping in mappings {
        if mapping.id == ignore_id {
            continue;
        }
        let existing = mapping.mount_path.as_str();
        if existing == new_mount_path {
            return Err(AppError::conflict("路径映射重复"));
        }
        if mount_paths_overlap(existing, new_mount_path) {
            return Err(AppError::conflict("路径映射冲突"));
        }
    }

    Ok(())
}

fn sorted_mappings(mappings: &[PathMapping]) -> Vec<PathMapping> {
    let mut mappings = mappings.to_vec();
    mappings.sort_by_key(|mapping| (mapping.order.unwrap_or(0), mapping.id.unwrap_or(0)));
    mappings
}

fn next_id(mappings: &[PathMapping]) -> i64 {
    mappings
        .iter()
        .filter_map(|mapping| mapping.id)
        .max()
        .unwrap_or(0)
        + 1
}

fn mount_paths_overlap(left: &str, right: &str) -> bool {
    is_same_or_child(left, right) || is_same_or_child(right, left)
}

fn is_same_or_child(path: &str, parent: &str) -> bool {
    parent == "/" || path == parent || path.starts_with(&format!("{parent}/"))
}

#[cfg(test)]
mod tests {
    use super::normalize_mount_path;

    #[test]
    fn normalizes_mount_path() {
        assert_eq!(normalize_mount_path("repo//src/").unwrap(), "/repo/src");
        assert_eq!(normalize_mount_path("/").unwrap(), "/");
    }

    #[test]
    fn rejects_parent_segments() {
        assert!(normalize_mount_path("/../secret").is_err());
    }
}
