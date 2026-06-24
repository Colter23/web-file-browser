use std::{path::PathBuf, sync::Arc};

use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        CreateFavoriteRequest, FavoriteItem, FavoriteResponse, ReorderFavoritesRequest,
        UpdateFavoriteRequest,
    },
    services::path_resolver::{self, MappingSnapshot, ensure_folder},
};

#[derive(Clone)]
pub struct FavoriteService {
    file_path: Arc<PathBuf>,
    items: Arc<RwLock<Vec<FavoriteItem>>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FavoriteStoreFile {
    #[serde(default)]
    items: Vec<FavoriteItem>,
}

impl FavoriteService {
    pub async fn load(file_path: PathBuf) -> Result<Self, AppError> {
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut items = match tokio::fs::read_to_string(&file_path).await {
            Ok(text) if text.trim().is_empty() => Vec::new(),
            Ok(text) => serde_json::from_str::<FavoriteStoreFile>(&text)?.items,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(error) => return Err(error.into()),
        };
        sort_items(&mut items);

        Ok(Self {
            file_path: Arc::new(file_path),
            items: Arc::new(RwLock::new(items)),
        })
    }

    pub async fn list(
        &self,
        snapshot: Arc<MappingSnapshot>,
        check: bool,
    ) -> Result<Vec<FavoriteResponse>, AppError> {
        let items = self.items.read().await.clone();
        tokio::task::spawn_blocking(move || {
            items
                .iter()
                .map(|item| response_from_item(&snapshot, item, check))
                .collect()
        })
        .await?
    }

    pub async fn create(
        &self,
        snapshot: Arc<MappingSnapshot>,
        request: CreateFavoriteRequest,
    ) -> Result<FavoriteResponse, AppError> {
        let resolved = path_resolver::resolve_existing(snapshot.clone(), request.path).await?;
        ensure_folder(&resolved.real_path, &resolved.virtual_path)?;

        let name = match request.name {
            Some(name) => normalize_favorite_name(name)?,
            None => default_favorite_name(&resolved),
        };
        let relative_path = resolved.relative_parts.join("/");
        let mut items = self.items.write().await;
        let candidate = FavoriteItem {
            id: Uuid::new_v4().to_string(),
            mount_id: resolved.mapping.id,
            mount_path: resolved.mapping.mount_path,
            relative_path,
            name,
            order: request.order.unwrap_or_else(|| next_order(&items)),
            created_at: current_time_text()?,
        };

        if items.iter().any(|item| same_target(item, &candidate)) {
            return Err(AppError::conflict("该文件夹已经在收藏夹中"));
        }

        items.push(candidate.clone());
        sort_items(&mut items);
        save_items(&self.file_path, &items).await?;
        response_from_item(&snapshot, &candidate, false)
    }

    pub async fn update(
        &self,
        snapshot: Arc<MappingSnapshot>,
        id: String,
        request: UpdateFavoriteRequest,
    ) -> Result<FavoriteResponse, AppError> {
        let mut items = self.items.write().await;
        let Some(item) = items.iter_mut().find(|item| item.id == id) else {
            return Err(AppError::not_found("收藏夹条目不存在"));
        };

        if let Some(name) = request.name {
            item.name = normalize_favorite_name(name)?;
        }
        if let Some(order) = request.order {
            item.order = order;
        }

        let updated = item.clone();
        sort_items(&mut items);
        save_items(&self.file_path, &items).await?;
        response_from_item(&snapshot, &updated, false)
    }

    pub async fn delete(&self, id: String) -> Result<(), AppError> {
        let mut items = self.items.write().await;
        let before_len = items.len();
        items.retain(|item| item.id != id);
        if items.len() == before_len {
            return Err(AppError::not_found("收藏夹条目不存在"));
        }
        save_items(&self.file_path, &items).await
    }

    pub async fn reorder(&self, request: ReorderFavoritesRequest) -> Result<(), AppError> {
        let mut items = self.items.write().await;
        for requested in &request.items {
            if !items.iter().any(|item| item.id == requested.id) {
                return Err(AppError::not_found(format!(
                    "收藏夹条目不存在: {}",
                    requested.id
                )));
            }
        }

        for requested in request.items {
            if let Some(item) = items.iter_mut().find(|item| item.id == requested.id) {
                item.order = requested.order;
            }
        }
        sort_items(&mut items);
        save_items(&self.file_path, &items).await
    }
}

async fn save_items(file_path: &PathBuf, items: &[FavoriteItem]) -> Result<(), AppError> {
    if let Some(parent) = file_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let text = serde_json::to_vec_pretty(&FavoriteStoreFile {
        items: items.to_vec(),
    })?;
    tokio::fs::write(file_path, text).await?;
    Ok(())
}

fn response_from_item(
    snapshot: &MappingSnapshot,
    item: &FavoriteItem,
    check: bool,
) -> Result<FavoriteResponse, AppError> {
    let mount_path = current_mount_path(snapshot, item).unwrap_or_else(|| item.mount_path.clone());
    let path = join_virtual_relative(&mount_path, &item.relative_path);
    let missing = check.then(|| !favorite_folder_exists(snapshot, &path));
    Ok(FavoriteResponse {
        id: item.id.clone(),
        mount_id: item.mount_id,
        mount_path,
        relative_path: item.relative_path.clone(),
        path,
        name: item.name.clone(),
        order: item.order,
        created_at: item.created_at.clone(),
        missing,
    })
}

fn current_mount_path(snapshot: &MappingSnapshot, item: &FavoriteItem) -> Option<String> {
    if let Some(mount_id) = item.mount_id
        && let Some(mapping) = snapshot
            .mappings
            .iter()
            .find(|mapping| mapping.id == Some(mount_id))
    {
        return Some(mapping.mount_path.clone());
    }

    snapshot
        .mappings
        .iter()
        .find(|mapping| mapping.mount_path == item.mount_path)
        .map(|mapping| mapping.mount_path.clone())
}

fn favorite_folder_exists(snapshot: &MappingSnapshot, path: &str) -> bool {
    path_resolver::resolve_existing_sync(snapshot, path)
        .and_then(|resolved| ensure_folder(&resolved.real_path, &resolved.virtual_path))
        .is_ok()
}

fn join_virtual_relative(mount_path: &str, relative_path: &str) -> String {
    if relative_path.is_empty() {
        mount_path.to_string()
    } else if mount_path == "/" {
        format!("/{relative_path}")
    } else {
        format!("{}/{}", mount_path.trim_end_matches('/'), relative_path)
    }
}

fn default_favorite_name(resolved: &path_resolver::ResolvedPath) -> String {
    resolved
        .relative_parts
        .last()
        .cloned()
        .or_else(|| {
            resolved
                .mapping
                .mount_path
                .trim_matches('/')
                .rsplit('/')
                .find(|part| !part.is_empty())
                .map(ToString::to_string)
        })
        .unwrap_or_else(|| "根目录".to_string())
}

fn normalize_favorite_name(name: String) -> Result<String, AppError> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::bad_request("收藏夹名称不能为空"));
    }
    if name.chars().count() > 128 {
        return Err(AppError::bad_request("收藏夹名称不能超过 128 个字符"));
    }
    Ok(name)
}

fn next_order(items: &[FavoriteItem]) -> i32 {
    items
        .iter()
        .map(|item| item.order)
        .max()
        .unwrap_or(0)
        .saturating_add(10)
}

fn same_target(left: &FavoriteItem, right: &FavoriteItem) -> bool {
    let same_mount = match (left.mount_id, right.mount_id) {
        (Some(left_id), Some(right_id)) => left_id == right_id,
        _ => left.mount_path == right.mount_path,
    };
    same_mount && left.relative_path == right.relative_path
}

fn sort_items(items: &mut [FavoriteItem]) {
    items.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then_with(|| left.created_at.cmp(&right.created_at))
            .then_with(|| left.id.cmp(&right.id))
    });
}

fn current_time_text() -> Result<String, AppError> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| AppError::internal(format!("生成收藏夹时间失败: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_mount_and_relative_path() {
        assert_eq!(join_virtual_relative("/docs", ""), "/docs");
        assert_eq!(join_virtual_relative("/docs", "a/b"), "/docs/a/b");
        assert_eq!(join_virtual_relative("/", "a/b"), "/a/b");
    }

    #[test]
    fn rejects_empty_favorite_name() {
        assert!(normalize_favorite_name("  ".to_string()).is_err());
    }
}
