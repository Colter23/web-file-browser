use std::{
    cmp::Reverse,
    fs,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

use crate::{
    error::AppError,
    models::{FileInfo, FolderData, FolderNode, PathMapping},
    services::{path_display::display_path, reserved},
};

mod listing;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DirectoryDetail {
    #[default]
    Basic,
    Full,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DirectorySort {
    #[default]
    Name,
    Modified,
    Size,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SortOrder {
    #[default]
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum EntryFilter {
    #[default]
    All,
    File,
    Folder,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DirectoryListOptions {
    pub offset: usize,
    pub limit: Option<usize>,
    pub detail: DirectoryDetail,
    pub sort: DirectorySort,
    pub order: SortOrder,
    pub filter: EntryFilter,
    pub include_hidden: bool,
    pub include_total: bool,
}

impl DirectoryListOptions {
    pub fn validate(self) -> Result<Self, AppError> {
        if self.detail == DirectoryDetail::Basic && self.sort != DirectorySort::Name {
            return Err(AppError::bad_request(
                "detail=basic 仅支持 sort=name，按大小或修改时间排序请使用 detail=full",
            )
            .with_reason("DIRECTORY_BASIC_DETAIL_REQUIRES_NAME_SORT")
            .with_param("detail", "basic")
            .with_param("sort", "name"));
        }
        Ok(self)
    }
}

#[derive(Debug, Clone)]
pub struct MappingSnapshot {
    pub mappings: Vec<PathMapping>,
    mounts: Vec<MountEntry>,
    root: Option<FolderNode>,
}

#[derive(Debug, Clone)]
struct MountEntry {
    mapping: PathMapping,
    mount_parts: Vec<String>,
    root_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ResolvedPath {
    pub mapping: PathMapping,
    pub relative_parts: Vec<String>,
    pub virtual_path: String,
    pub real_path: PathBuf,
    pub mount_root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ResolvedParentPath {
    pub parent_virtual_path: String,
    pub parent_real_path: PathBuf,
    pub child_name: String,
    pub child_virtual_path: String,
}

pub enum MetadataEntry {
    Folder {
        data: FolderData,
        modified: Option<String>,
    },
    File(FileInfo),
}

impl MappingSnapshot {
    pub async fn build(mappings: Vec<PathMapping>) -> Result<Arc<Self>, AppError> {
        tokio::task::spawn_blocking(move || build_snapshot_sync(mappings)).await?
    }

    pub fn root_node(&self) -> Option<FolderNode> {
        self.root.clone()
    }
}

pub async fn metadata(
    snapshot: Arc<MappingSnapshot>,
    path: String,
    options: DirectoryListOptions,
) -> Result<MetadataEntry, AppError> {
    let options = options.validate()?;
    tokio::task::spawn_blocking(move || metadata_sync_with_options(&snapshot, &path, options))
        .await?
}

pub async fn basic_metadata_modified(
    snapshot: Arc<MappingSnapshot>,
    path: String,
) -> Result<Option<String>, AppError> {
    tokio::task::spawn_blocking(move || basic_metadata_modified_sync(&snapshot, &path)).await?
}

pub async fn resolve_existing(
    snapshot: Arc<MappingSnapshot>,
    path: String,
) -> Result<ResolvedPath, AppError> {
    tokio::task::spawn_blocking(move || resolve_existing_sync(&snapshot, &path)).await?
}

pub async fn resolve_existing_no_follow_final(
    snapshot: Arc<MappingSnapshot>,
    path: String,
) -> Result<ResolvedPath, AppError> {
    tokio::task::spawn_blocking(move || resolve_existing_no_follow_final_sync(&snapshot, &path))
        .await?
}

pub fn resolve_existing_sync(
    snapshot: &MappingSnapshot,
    path: &str,
) -> Result<ResolvedPath, AppError> {
    let parts = split_virtual_path(path)?;
    let virtual_path = virtual_path_from_parts(&parts);
    let (mount, relative_parts) =
        find_mount(snapshot, &parts).ok_or_else(|| not_found_path(&virtual_path))?;
    let real_path = secure_join_existing(&mount.root_path, &relative_parts, &virtual_path)?;

    Ok(ResolvedPath {
        mapping: mount.mapping.clone(),
        relative_parts,
        virtual_path,
        real_path,
        mount_root: mount.root_path.clone(),
    })
}

pub fn resolve_existing_no_follow_final_sync(
    snapshot: &MappingSnapshot,
    path: &str,
) -> Result<ResolvedPath, AppError> {
    let parts = split_virtual_path(path)?;
    let virtual_path = virtual_path_from_parts(&parts);
    let (mount, relative_parts) =
        find_mount(snapshot, &parts).ok_or_else(|| not_found_path(&virtual_path))?;
    let real_path =
        secure_join_existing_no_follow_final(&mount.root_path, &relative_parts, &virtual_path)?;

    Ok(ResolvedPath {
        mapping: mount.mapping.clone(),
        relative_parts,
        virtual_path,
        real_path,
        mount_root: mount.root_path.clone(),
    })
}

pub fn resolve_parent_for_child_sync(
    snapshot: &MappingSnapshot,
    child_path: &str,
) -> Result<ResolvedParentPath, AppError> {
    let parts = split_virtual_path(child_path)?;
    let Some(child_name) = parts.last().cloned() else {
        return Err(AppError::bad_request("目标路径不能为空").with_reason("TARGET_PATH_EMPTY"));
    };
    let parent_parts = parts[..parts.len() - 1].to_vec();
    let parent_virtual_path = virtual_path_from_parts(&parent_parts);
    let parent = resolve_existing_sync(snapshot, &parent_virtual_path)?;
    ensure_writable(&parent.mapping)?;
    ensure_folder(&parent.real_path, &parent.virtual_path)?;

    Ok(ResolvedParentPath {
        parent_virtual_path: parent.virtual_path.clone(),
        parent_real_path: parent.real_path,
        child_virtual_path: join_virtual_path(&parent_virtual_path, &child_name),
        child_name,
    })
}

pub fn split_virtual_path(path: &str) -> Result<Vec<String>, AppError> {
    let mut parts = Vec::new();
    for part in path.trim_matches('/').split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        validate_path_segment(part)?;
        parts.push(part.to_string());
    }
    Ok(parts)
}

pub fn virtual_path_from_parts(parts: &[String]) -> String {
    if parts.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", parts.join("/"))
    }
}

pub fn join_virtual_path(base: &str, name: &str) -> String {
    if base == "/" {
        format!("/{name}")
    } else {
        format!("{}/{}", base.trim_end_matches('/'), name)
    }
}

pub fn normalize_child_name(name: &str) -> Result<String, AppError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::bad_request("名称不能为空")
            .with_reason("ENTRY_NAME_EMPTY")
            .with_param("field", "name"));
    }
    validate_path_segment(name)?;
    Ok(name.to_string())
}

pub fn ensure_writable(mapping: &PathMapping) -> Result<(), AppError> {
    if mapping.writable {
        Ok(())
    } else {
        Err(AppError::forbidden("挂载点是只读模式").with_reason("MOUNT_READONLY"))
    }
}

pub fn ensure_not_mount_root(resolved: &ResolvedPath) -> Result<(), AppError> {
    if resolved.relative_parts.is_empty() {
        Err(AppError::bad_request("不能操作挂载根路径")
            .with_reason("MOUNT_ROOT_OPERATION_FORBIDDEN"))
    } else {
        Ok(())
    }
}

pub fn ensure_folder(path: &Path, virtual_path: &str) -> Result<(), AppError> {
    if path.is_dir() {
        Ok(())
    } else {
        Err(
            AppError::bad_request(format!("路径不是文件夹: {virtual_path}"))
                .with_reason("PATH_NOT_FOLDER")
                .with_param("path", virtual_path),
        )
    }
}

pub fn ensure_file(path: &Path, virtual_path: &str) -> Result<(), AppError> {
    if path.is_file() {
        Ok(())
    } else {
        Err(
            AppError::bad_request(format!("路径不是文件: {virtual_path}"))
                .with_reason("PATH_NOT_FILE")
                .with_param("path", virtual_path),
        )
    }
}

fn build_snapshot_sync(mut mappings: Vec<PathMapping>) -> Result<Arc<MappingSnapshot>, AppError> {
    mappings.sort_by_key(|mapping| (mapping.order.unwrap_or(0), mapping.id.unwrap_or(0)));

    let mut mounts = Vec::with_capacity(mappings.len());
    for mapping in &mut mappings {
        let root_path = Path::new(&mapping.folder_path)
            .canonicalize()
            .map_err(|_| {
                mapping_root_not_found(&mapping.folder_path)
                    .with_param("mappingMountPath", mapping.mount_path.clone())
            })?;
        mapping.folder_path = display_path(&root_path);
        mounts.push(MountEntry {
            mapping: mapping.clone(),
            mount_parts: mapping_parts(&mapping.mount_path),
            root_path,
        });
    }
    mounts.sort_by_key(|mount| Reverse(mount.mount_parts.len()));

    let root = build_root(&mappings);
    Ok(Arc::new(MappingSnapshot {
        mappings,
        mounts,
        root,
    }))
}

#[cfg(test)]
fn metadata_sync(snapshot: &MappingSnapshot, path: &str) -> Result<MetadataEntry, AppError> {
    metadata_sync_with_options(snapshot, path, DirectoryListOptions::default())
}

fn metadata_sync_with_options(
    snapshot: &MappingSnapshot,
    path: &str,
    options: DirectoryListOptions,
) -> Result<MetadataEntry, AppError> {
    let parts = split_virtual_path(path)?;
    let virtual_path = virtual_path_from_parts(&parts);

    if let Some((mount, relative_parts)) = find_mount(snapshot, &parts) {
        let real_path = secure_join_existing(&mount.root_path, &relative_parts, &virtual_path)?;
        let metadata = fs::metadata(&real_path).map_err(|_| not_found_path(&virtual_path))?;
        if metadata.is_dir() {
            let parent_path = join_virtual_parts(&mount.mapping.mount_path, &relative_parts);
            Ok(MetadataEntry::Folder {
                data: listing::list_real_folder(&real_path, &parent_path, options)?,
                modified: Some(listing::modified_to_string(&metadata)),
            })
        } else {
            Ok(MetadataEntry::File(listing::file_info_from_path(
                &real_path,
                &virtual_path,
                &metadata,
            )))
        }
    } else {
        let Some(root) = &snapshot.root else {
            if !parts.is_empty() {
                return Err(not_found_path(&virtual_path));
            }
            return Ok(MetadataEntry::Folder {
                data: FolderData::full(virtual_path, Vec::new(), Vec::new()),
                modified: None,
            });
        };
        let Some(FolderNode::Virtual { children, .. }) = resolve_virtual_node(root, &parts) else {
            return Err(not_found_path(&virtual_path));
        };

        Ok(MetadataEntry::Folder {
            data: listing::list_virtual_folder(children, virtual_path, options),
            modified: None,
        })
    }
}

fn basic_metadata_modified_sync(
    snapshot: &MappingSnapshot,
    path: &str,
) -> Result<Option<String>, AppError> {
    let parts = split_virtual_path(path)?;
    let virtual_path = virtual_path_from_parts(&parts);

    if let Some((mount, relative_parts)) = find_mount(snapshot, &parts) {
        let real_path = secure_join_existing(&mount.root_path, &relative_parts, &virtual_path)?;
        let metadata = fs::metadata(&real_path).map_err(|_| not_found_path(&virtual_path))?;
        return Ok(Some(listing::modified_to_string(&metadata)));
    }

    let Some(root) = &snapshot.root else {
        if !parts.is_empty() {
            return Err(not_found_path(&virtual_path));
        }
        return Ok(None);
    };
    if resolve_virtual_node(root, &parts).is_none() {
        return Err(not_found_path(&virtual_path));
    }
    Ok(None)
}

fn find_mount<'a>(
    snapshot: &'a MappingSnapshot,
    parts: &[String],
) -> Option<(&'a MountEntry, Vec<String>)> {
    snapshot
        .mounts
        .iter()
        .find(|mount| is_prefix(&mount.mount_parts, parts))
        .map(|mount| (mount, parts[mount.mount_parts.len()..].to_vec()))
}

fn secure_join_existing(
    root: &Path,
    relative_parts: &[String],
    virtual_path: &str,
) -> Result<PathBuf, AppError> {
    ensure_not_reserved_relative_path(relative_parts)?;
    let mut target = root.to_path_buf();
    for part in relative_parts {
        validate_path_segment(part)?;
        target.push(part);
    }

    let target = target
        .canonicalize()
        .map_err(|_| not_found_path(virtual_path))?;
    if !target.starts_with(root) {
        return Err(path_outside_mount());
    }
    Ok(target)
}

fn secure_join_existing_no_follow_final(
    root: &Path,
    relative_parts: &[String],
    virtual_path: &str,
) -> Result<PathBuf, AppError> {
    ensure_not_reserved_relative_path(relative_parts)?;
    let mut target = root.to_path_buf();
    for part in relative_parts {
        validate_path_segment(part)?;
        target.push(part);
    }

    let metadata = fs::symlink_metadata(&target).map_err(|_| not_found_path(virtual_path))?;
    if metadata.file_type().is_symlink() {
        let parent = target
            .parent()
            .unwrap_or(root)
            .canonicalize()
            .map_err(|_| not_found_path(virtual_path))?;
        if !parent.starts_with(root) {
            return Err(path_outside_mount());
        }
        return Ok(target);
    }

    let target = target
        .canonicalize()
        .map_err(|_| not_found_path(virtual_path))?;
    if !target.starts_with(root) {
        return Err(path_outside_mount());
    }
    Ok(target)
}

fn ensure_not_reserved_relative_path(relative_parts: &[String]) -> Result<(), AppError> {
    if relative_parts
        .first()
        .is_some_and(|part| reserved::is_mount_trash_dir_name(part))
    {
        return Err(AppError::bad_request("不能访问应用内部回收站目录")
            .with_reason("RESERVED_TRASH_PATH_FORBIDDEN"));
    }
    Ok(())
}

fn validate_path_segment(part: &str) -> Result<(), AppError> {
    if part == ".." || part.contains('\\') || part.contains('/') {
        return Err(AppError::bad_request("路径不能包含 .. 或路径分隔符")
            .with_reason("PATH_SEGMENT_INVALID")
            .with_param("segment", part));
    }
    let part_path = Path::new(part);
    if part_path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(AppError::bad_request("路径包含非法片段")
            .with_reason("PATH_SEGMENT_INVALID")
            .with_param("segment", part));
    }
    Ok(())
}

fn not_found_path(path: &str) -> AppError {
    AppError::not_found(format!("查无此路径: {path}"))
        .with_reason("PATH_NOT_FOUND")
        .with_param("path", path)
}

fn path_outside_mount() -> AppError {
    AppError::bad_request("路径越界").with_reason("PATH_OUTSIDE_MOUNT")
}

fn mapping_root_not_found(path: &str) -> AppError {
    AppError::bad_request(format!("挂载本地目录不存在或不可访问: {path}"))
        .with_reason("MAPPING_FOLDER_PATH_NOT_FOUND")
        .with_param("path", path)
}

fn mapping_parts(path: &str) -> Vec<String> {
    path.trim_matches('/')
        .split('/')
        .filter(|part| !part.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn is_prefix(prefix: &[String], parts: &[String]) -> bool {
    prefix.len() <= parts.len()
        && prefix
            .iter()
            .zip(parts.iter())
            .all(|(left, right)| left == right)
}

fn build_root(mappings: &[PathMapping]) -> Option<FolderNode> {
    let mut root = None;
    for mapping in mappings {
        if mapping.mount_path == "/" {
            return Some(FolderNode::Real {
                name: "root".to_string(),
                path: "/".to_string(),
                real_path: mapping.folder_path.clone(),
            });
        }

        if root.is_none() {
            root = Some(FolderNode::Virtual {
                name: "root".to_string(),
                path: "/".to_string(),
                children: Vec::new(),
            });
        }

        if let Some(root) = root.as_mut() {
            add_mount_path(root, mapping);
        }
    }
    root
}

fn add_mount_path(root: &mut FolderNode, mapping: &PathMapping) {
    let parts = mapping_parts(&mapping.mount_path);
    add_mount_path_inner(root, &parts, mapping);
}

fn add_mount_path_inner(current: &mut FolderNode, parts: &[String], mapping: &PathMapping) {
    if parts.is_empty() {
        return;
    }

    let FolderNode::Virtual { path, children, .. } = current else {
        return;
    };

    let child_name = &parts[0];
    let child_path = join_virtual_path(path, child_name);
    if parts.len() == 1 {
        children.push(FolderNode::Real {
            name: child_name.to_string(),
            path: child_path,
            real_path: mapping.folder_path.clone(),
        });
        return;
    }

    let child_index = match children
        .iter()
        .position(|child| matches!(child, FolderNode::Virtual { name, .. } if name == child_name))
    {
        Some(index) => index,
        None => {
            children.push(FolderNode::Virtual {
                name: child_name.to_string(),
                path: child_path,
                children: Vec::new(),
            });
            children.len() - 1
        }
    };

    add_mount_path_inner(&mut children[child_index], &parts[1..], mapping);
}

fn resolve_virtual_node<'a>(root: &'a FolderNode, parts: &[String]) -> Option<&'a FolderNode> {
    let mut current = root;
    for part in parts {
        let FolderNode::Virtual { children, .. } = current else {
            return None;
        };
        current = children.iter().find(|child| child.name() == part)?;
    }
    Some(current)
}

fn join_virtual_parts(base: &str, parts: &[String]) -> String {
    parts.iter().fold(base.to_string(), |path, part| {
        join_virtual_path(&path, part)
    })
}

#[cfg(test)]
mod tests;
