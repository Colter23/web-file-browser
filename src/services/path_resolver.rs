use std::{
    cmp::{Ordering, Reverse},
    collections::BinaryHeap,
    fs,
    path::{Component, Path, PathBuf},
    sync::Arc,
    time::UNIX_EPOCH,
};

use crate::{
    error::AppError,
    models::{
        EntryKind, FileInfo, FolderData, FolderInfo, FolderNode, FolderPageInfo, PathMapping,
    },
};

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
    fn validate(self) -> Result<Self, AppError> {
        if self.detail == DirectoryDetail::Basic && self.sort != DirectorySort::Name {
            return Err(AppError::bad_request(
                "detail=basic 仅支持 sort=name，按大小或修改时间排序请使用 detail=full",
            ));
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
    let (mount, relative_parts) = find_mount(snapshot, &parts)
        .ok_or_else(|| AppError::not_found(format!("查无此路径: {virtual_path}")))?;
    let real_path = secure_join_existing(&mount.root_path, &relative_parts, &virtual_path)?;

    Ok(ResolvedPath {
        mapping: mount.mapping.clone(),
        relative_parts,
        virtual_path,
        real_path,
    })
}

pub fn resolve_existing_no_follow_final_sync(
    snapshot: &MappingSnapshot,
    path: &str,
) -> Result<ResolvedPath, AppError> {
    let parts = split_virtual_path(path)?;
    let virtual_path = virtual_path_from_parts(&parts);
    let (mount, relative_parts) = find_mount(snapshot, &parts)
        .ok_or_else(|| AppError::not_found(format!("查无此路径: {virtual_path}")))?;
    let real_path =
        secure_join_existing_no_follow_final(&mount.root_path, &relative_parts, &virtual_path)?;

    Ok(ResolvedPath {
        mapping: mount.mapping.clone(),
        relative_parts,
        virtual_path,
        real_path,
    })
}

pub fn resolve_parent_for_child_sync(
    snapshot: &MappingSnapshot,
    child_path: &str,
) -> Result<ResolvedParentPath, AppError> {
    let parts = split_virtual_path(child_path)?;
    let Some(child_name) = parts.last().cloned() else {
        return Err(AppError::bad_request("目标路径不能为空"));
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
        return Err(AppError::bad_request("名称不能为空"));
    }
    validate_path_segment(name)?;
    Ok(name.to_string())
}

pub fn ensure_writable(mapping: &PathMapping) -> Result<(), AppError> {
    if mapping.writable {
        Ok(())
    } else {
        Err(AppError::forbidden("挂载点是只读模式"))
    }
}

pub fn ensure_not_mount_root(resolved: &ResolvedPath) -> Result<(), AppError> {
    if resolved.relative_parts.is_empty() {
        Err(AppError::bad_request("不能操作挂载根路径"))
    } else {
        Ok(())
    }
}

pub fn ensure_folder(path: &Path, virtual_path: &str) -> Result<(), AppError> {
    if path.is_dir() {
        Ok(())
    } else {
        Err(AppError::bad_request(format!(
            "路径不是文件夹: {virtual_path}"
        )))
    }
}

pub fn ensure_file(path: &Path, virtual_path: &str) -> Result<(), AppError> {
    if path.is_file() {
        Ok(())
    } else {
        Err(AppError::bad_request(format!(
            "路径不是文件: {virtual_path}"
        )))
    }
}

fn build_snapshot_sync(mut mappings: Vec<PathMapping>) -> Result<Arc<MappingSnapshot>, AppError> {
    mappings.sort_by_key(|mapping| (mapping.order.unwrap_or(0), mapping.id.unwrap_or(0)));

    let mut mounts = Vec::with_capacity(mappings.len());
    for mapping in &mut mappings {
        let root_path = Path::new(&mapping.folder_path)
            .canonicalize()
            .map_err(|_| AppError::bad_request(format!("查无此路径: {}", mapping.folder_path)))?;
        mapping.folder_path = root_path.to_string_lossy().to_string();
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
        let metadata = fs::metadata(&real_path)
            .map_err(|_| AppError::not_found(format!("查无此路径: {virtual_path}")))?;
        if metadata.is_dir() {
            let parent_path = join_virtual_parts(&mount.mapping.mount_path, &relative_parts);
            Ok(MetadataEntry::Folder {
                data: list_real_folder(&real_path, &parent_path, options)?,
                modified: Some(modified_to_string(&metadata)),
            })
        } else {
            Ok(MetadataEntry::File(file_info_from_path(
                &real_path,
                &virtual_path,
                &metadata,
            )))
        }
    } else {
        let Some(root) = &snapshot.root else {
            if !parts.is_empty() {
                return Err(AppError::not_found(format!("查无此路径: {virtual_path}")));
            }
            return Ok(MetadataEntry::Folder {
                data: FolderData::full(virtual_path, Vec::new(), Vec::new()),
                modified: None,
            });
        };
        let Some(FolderNode::Virtual { children, .. }) = resolve_virtual_node(root, &parts) else {
            return Err(AppError::not_found(format!("查无此路径: {virtual_path}")));
        };

        Ok(MetadataEntry::Folder {
            data: list_virtual_folder(children, virtual_path, options),
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
        let metadata = fs::metadata(&real_path)
            .map_err(|_| AppError::not_found(format!("查无此路径: {virtual_path}")))?;
        return Ok(Some(modified_to_string(&metadata)));
    }

    let Some(root) = &snapshot.root else {
        if !parts.is_empty() {
            return Err(AppError::not_found(format!("查无此路径: {virtual_path}")));
        }
        return Ok(None);
    };
    if resolve_virtual_node(root, &parts).is_none() {
        return Err(AppError::not_found(format!("查无此路径: {virtual_path}")));
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
    let mut target = root.to_path_buf();
    for part in relative_parts {
        validate_path_segment(part)?;
        target.push(part);
    }

    let target = target
        .canonicalize()
        .map_err(|_| AppError::not_found(format!("查无此路径: {virtual_path}")))?;
    if !target.starts_with(root) {
        return Err(AppError::bad_request("路径越界"));
    }
    Ok(target)
}

fn secure_join_existing_no_follow_final(
    root: &Path,
    relative_parts: &[String],
    virtual_path: &str,
) -> Result<PathBuf, AppError> {
    let mut target = root.to_path_buf();
    for part in relative_parts {
        validate_path_segment(part)?;
        target.push(part);
    }

    let metadata = fs::symlink_metadata(&target)
        .map_err(|_| AppError::not_found(format!("查无此路径: {virtual_path}")))?;
    if metadata.file_type().is_symlink() {
        let parent = target
            .parent()
            .unwrap_or(root)
            .canonicalize()
            .map_err(|_| AppError::not_found(format!("查无此路径: {virtual_path}")))?;
        if !parent.starts_with(root) {
            return Err(AppError::bad_request("路径越界"));
        }
        return Ok(target);
    }

    let target = target
        .canonicalize()
        .map_err(|_| AppError::not_found(format!("查无此路径: {virtual_path}")))?;
    if !target.starts_with(root) {
        return Err(AppError::bad_request("路径越界"));
    }
    Ok(target)
}

fn validate_path_segment(part: &str) -> Result<(), AppError> {
    if part == ".." || part.contains('\\') || part.contains('/') {
        return Err(AppError::bad_request("路径不能包含 .. 或路径分隔符"));
    }
    let part_path = Path::new(part);
    if part_path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(AppError::bad_request("路径包含非法片段"));
    }
    Ok(())
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

fn list_real_folder(
    path: &Path,
    parent_path: &str,
    options: DirectoryListOptions,
) -> Result<FolderData, AppError> {
    if options.detail == DirectoryDetail::Basic && options.sort == DirectorySort::Name {
        return list_real_folder_basic(path, parent_path, options);
    }

    list_real_folder_full(path, parent_path, options)
}

#[derive(Debug, Clone)]
struct LightEntry {
    name: String,
    kind: EntryKind,
}

#[derive(Debug)]
struct DetailedEntry {
    light: LightEntry,
    metadata: fs::Metadata,
}

#[derive(Debug, Default)]
struct LightReadResult {
    entries: Vec<LightEntry>,
    total_entries: usize,
    folder_total: usize,
    file_total: usize,
}

#[derive(Debug, Clone)]
struct LightEntryHeapItem {
    entry: LightEntry,
    order: SortOrder,
}

impl PartialEq for LightEntryHeapItem {
    fn eq(&self, other: &Self) -> bool {
        self.entry.kind == other.entry.kind && self.entry.name == other.entry.name
    }
}

impl Eq for LightEntryHeapItem {}

impl PartialOrd for LightEntryHeapItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LightEntryHeapItem {
    fn cmp(&self, other: &Self) -> Ordering {
        cmp_light_entries(&self.entry, &other.entry, self.order)
    }
}

fn read_light_entries(
    path: &Path,
    options: DirectoryListOptions,
) -> Result<LightReadResult, AppError> {
    read_light_entries_with_window(path, options, None)
}

fn read_light_entries_page(
    path: &Path,
    options: DirectoryListOptions,
) -> Result<LightReadResult, AppError> {
    read_light_entries_with_window(
        path,
        options,
        options
            .limit
            .map(|limit| options.offset.saturating_add(limit)),
    )
}

fn read_light_entries_with_window(
    path: &Path,
    options: DirectoryListOptions,
    keep_window: Option<usize>,
) -> Result<LightReadResult, AppError> {
    let mut result = LightReadResult::default();
    let mut heap = keep_window.map(|_| BinaryHeap::<LightEntryHeapItem>::new());
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !options.include_hidden && name.starts_with('.') {
            continue;
        }
        let kind = if entry.file_type()?.is_dir() {
            EntryKind::Folder
        } else {
            EntryKind::File
        };
        if !filter_allows(options.filter, kind) {
            continue;
        }

        result.total_entries += 1;
        match kind {
            EntryKind::Folder => result.folder_total += 1,
            EntryKind::File => result.file_total += 1,
        }

        let light_entry = LightEntry { name, kind };
        match (keep_window, heap.as_mut()) {
            (Some(0), _) => {}
            (Some(keep), Some(heap)) if heap.len() < keep => {
                heap.push(LightEntryHeapItem {
                    entry: light_entry,
                    order: options.order,
                });
            }
            (Some(_), Some(heap)) => {
                let should_keep = heap.peek().is_some_and(|worst| {
                    cmp_light_entries(&light_entry, &worst.entry, options.order) == Ordering::Less
                });
                if should_keep {
                    heap.pop();
                    heap.push(LightEntryHeapItem {
                        entry: light_entry,
                        order: options.order,
                    });
                }
            }
            _ => result.entries.push(light_entry),
        }
    }
    if let Some(heap) = heap {
        result.entries = heap.into_iter().map(|item| item.entry).collect();
    }
    Ok(result)
}

fn list_real_folder_basic(
    path: &Path,
    parent_path: &str,
    options: DirectoryListOptions,
) -> Result<FolderData, AppError> {
    let mut result = read_light_entries_page(path, options)?;
    sort_light_entries(&mut result.entries, options.order);
    let has_more = has_more(result.total_entries, options);
    let totals = count_totals_if_requested(&result, options);
    let entries = page_entries(result.entries, options);
    let mut folder = Vec::new();
    let mut file = Vec::new();
    for entry in entries {
        match entry.kind {
            EntryKind::Folder => folder.push(folder_info_basic(&entry, parent_path)),
            EntryKind::File => file.push(file_info_basic(&entry, parent_path)),
        }
    }

    Ok(folder_data(
        parent_path.to_string(),
        folder,
        file,
        totals,
        has_more,
        options,
    ))
}

fn list_real_folder_full(
    path: &Path,
    parent_path: &str,
    options: DirectoryListOptions,
) -> Result<FolderData, AppError> {
    if options.sort == DirectorySort::Name {
        let mut result = read_light_entries_page(path, options)?;
        sort_light_entries(&mut result.entries, options.order);
        let has_more = has_more(result.total_entries, options);
        let totals = count_totals_if_requested(&result, options);
        let entries = page_entries(result.entries, options)
            .into_iter()
            .map(|entry| detailed_entry(path, entry))
            .collect::<Result<Vec<_>, std::io::Error>>()?;
        return Ok(folder_data_from_detailed_entries(
            parent_path.to_string(),
            entries,
            totals,
            has_more,
            options,
        ));
    }

    let result = read_light_entries(path, options)?;
    let has_more = has_more(result.total_entries, options);
    let totals = count_totals_if_requested(&result, options);
    let mut entries = result
        .entries
        .into_iter()
        .map(|entry| detailed_entry(path, entry))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    sort_detailed_entries(&mut entries, options.sort, options.order);
    let entries = page_entries(entries, options);

    Ok(folder_data_from_detailed_entries(
        parent_path.to_string(),
        entries,
        totals,
        has_more,
        options,
    ))
}

fn detailed_entry(parent_path: &Path, entry: LightEntry) -> Result<DetailedEntry, std::io::Error> {
    let metadata = fs::metadata(parent_path.join(&entry.name))?;
    Ok(DetailedEntry {
        light: entry,
        metadata,
    })
}

fn folder_data_from_detailed_entries(
    parent_path: String,
    entries: Vec<DetailedEntry>,
    totals: Option<(usize, usize)>,
    has_more: bool,
    options: DirectoryListOptions,
) -> FolderData {
    let mut folder = Vec::new();
    let mut file = Vec::new();
    for entry in entries {
        match entry.light.kind {
            EntryKind::Folder => folder.push(folder_info_from_detailed_entry(&entry, &parent_path)),
            EntryKind::File => file.push(file_info_from_detailed_entry(&entry, &parent_path)),
        }
    }

    folder_data(parent_path, folder, file, totals, has_more, options)
}

fn folder_data(
    path: String,
    folder: Vec<FolderInfo>,
    file: Vec<FileInfo>,
    totals: Option<(usize, usize)>,
    has_more: bool,
    options: DirectoryListOptions,
) -> FolderData {
    if let Some(limit) = options.limit {
        let (folder_total, file_total) = totals
            .map(|(folder_total, file_total)| (Some(folder_total), Some(file_total)))
            .unwrap_or((None, None));
        FolderData::paged(
            path,
            folder,
            file,
            FolderPageInfo {
                folder_total,
                file_total,
                offset: options.offset,
                limit,
                has_more,
            },
        )
    } else {
        FolderData::full(path, folder, file)
    }
}

fn list_virtual_folder(
    children: &[FolderNode],
    virtual_path: String,
    options: DirectoryListOptions,
) -> FolderData {
    let mut folder = children
        .iter()
        .filter(|node| {
            options.filter != EntryFilter::File
                && (options.include_hidden || !node.name().starts_with('.'))
        })
        .map(|node| folder_info_from_node(node, options.detail))
        .collect::<Vec<_>>();
    folder.sort_by(|left, right| left.name.cmp(&right.name));
    if options.order == SortOrder::Desc {
        folder.reverse();
    }
    if let Some(limit) = options.limit {
        let has_more = has_more(folder.len(), options);
        let folder_total = options.include_total.then_some(folder.len());
        let folder = folder
            .into_iter()
            .skip(options.offset)
            .take(limit)
            .collect::<Vec<_>>();
        FolderData::paged(
            virtual_path,
            folder,
            Vec::new(),
            FolderPageInfo {
                folder_total,
                file_total: options.include_total.then_some(0),
                offset: options.offset,
                limit,
                has_more,
            },
        )
    } else {
        FolderData::full(virtual_path, folder, Vec::new())
    }
}

fn folder_info_from_node(node: &FolderNode, detail: DirectoryDetail) -> FolderInfo {
    match node {
        FolderNode::Virtual { name, path, .. } => FolderInfo {
            name: name.clone(),
            path: path.clone(),
            modified: String::new(),
            entry_type: EntryKind::Folder.as_str().to_string(),
        },
        FolderNode::Real {
            name,
            path,
            real_path,
        } => {
            let modified = if detail == DirectoryDetail::Full {
                fs::metadata(real_path)
                    .ok()
                    .map(|metadata| modified_to_string(&metadata))
                    .unwrap_or_default()
            } else {
                String::new()
            };
            FolderInfo {
                name: name.clone(),
                path: path.clone(),
                modified,
                entry_type: EntryKind::Folder.as_str().to_string(),
            }
        }
    }
}

fn folder_info_basic(entry: &LightEntry, parent_path: &str) -> FolderInfo {
    FolderInfo {
        name: entry.name.clone(),
        path: join_virtual_path(parent_path, &entry.name),
        modified: String::new(),
        entry_type: EntryKind::Folder.as_str().to_string(),
    }
}

fn file_info_basic(entry: &LightEntry, parent_path: &str) -> FileInfo {
    let extension = Path::new(&entry.name)
        .extension()
        .map(|extension| extension.to_string_lossy().to_string())
        .unwrap_or_default();
    FileInfo {
        name: entry.name.clone(),
        path: join_virtual_path(parent_path, &entry.name),
        modified: String::new(),
        size: 0,
        extension,
        entry_type: EntryKind::File.as_str().to_string(),
    }
}

fn folder_info_from_detailed_entry(entry: &DetailedEntry, parent_path: &str) -> FolderInfo {
    FolderInfo {
        name: entry.light.name.clone(),
        path: join_virtual_path(parent_path, &entry.light.name),
        modified: modified_to_string(&entry.metadata),
        entry_type: EntryKind::Folder.as_str().to_string(),
    }
}

fn file_info_from_detailed_entry(entry: &DetailedEntry, parent_path: &str) -> FileInfo {
    let extension = Path::new(&entry.light.name)
        .extension()
        .map(|extension| extension.to_string_lossy().to_string())
        .unwrap_or_default();
    FileInfo {
        name: entry.light.name.clone(),
        path: join_virtual_path(parent_path, &entry.light.name),
        modified: modified_to_string(&entry.metadata),
        size: entry.metadata.len(),
        extension,
        entry_type: EntryKind::File.as_str().to_string(),
    }
}

fn file_info_from_path(path: &Path, virtual_path: &str, metadata: &fs::Metadata) -> FileInfo {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default();
    let extension = path
        .extension()
        .map(|extension| extension.to_string_lossy().to_string())
        .unwrap_or_default();

    FileInfo {
        name,
        path: virtual_path.to_string(),
        modified: modified_to_string(metadata),
        size: metadata.len(),
        extension,
        entry_type: EntryKind::File.as_str().to_string(),
    }
}

fn sort_light_entries(entries: &mut [LightEntry], order: SortOrder) {
    entries.sort_by(|left, right| cmp_light_entries(left, right, order));
}

fn cmp_light_entries(left: &LightEntry, right: &LightEntry, order: SortOrder) -> Ordering {
    let ordering = kind_order(left.kind)
        .cmp(&kind_order(right.kind))
        .then_with(|| left.name.cmp(&right.name));
    if order == SortOrder::Desc {
        ordering.reverse()
    } else {
        ordering
    }
}

fn sort_detailed_entries(entries: &mut [DetailedEntry], sort: DirectorySort, order: SortOrder) {
    entries.sort_by(|left, right| {
        let ordering = kind_order(left.light.kind)
            .cmp(&kind_order(right.light.kind))
            .then_with(|| match sort {
                DirectorySort::Name => left.light.name.cmp(&right.light.name),
                DirectorySort::Modified => left
                    .metadata
                    .modified()
                    .ok()
                    .cmp(&right.metadata.modified().ok()),
                DirectorySort::Size => left.metadata.len().cmp(&right.metadata.len()),
            });
        if order == SortOrder::Desc {
            ordering.reverse()
        } else {
            ordering
        }
    });
}

fn kind_order(kind: EntryKind) -> u8 {
    match kind {
        EntryKind::Folder => 0,
        EntryKind::File => 1,
    }
}

fn filter_allows(filter: EntryFilter, kind: EntryKind) -> bool {
    matches!(
        (filter, kind),
        (EntryFilter::All, _)
            | (EntryFilter::File, EntryKind::File)
            | (EntryFilter::Folder, EntryKind::Folder)
    )
}

fn count_totals_if_requested(
    result: &LightReadResult,
    options: DirectoryListOptions,
) -> Option<(usize, usize)> {
    options
        .include_total
        .then_some((result.folder_total, result.file_total))
}

fn has_more(total_entries: usize, options: DirectoryListOptions) -> bool {
    options
        .limit
        .map(|limit| options.offset.saturating_add(limit) < total_entries)
        .unwrap_or(false)
}

fn page_entries<T>(entries: Vec<T>, options: DirectoryListOptions) -> Vec<T> {
    if let Some(limit) = options.limit {
        entries
            .into_iter()
            .skip(options.offset)
            .take(limit)
            .collect()
    } else {
        entries
    }
}

fn join_virtual_parts(base: &str, parts: &[String]) -> String {
    parts.iter().fold(base.to_string(), |path, part| {
        join_virtual_path(&path, part)
    })
}

fn modified_to_string(metadata: &fs::Metadata) -> String {
    metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::models::PathMapping;

    use super::{
        DirectoryDetail, DirectoryListOptions, MappingSnapshot, MetadataEntry,
        basic_metadata_modified_sync, metadata_sync, metadata_sync_with_options, page_entries,
        read_light_entries_page, resolve_existing_no_follow_final_sync, resolve_existing_sync,
        sort_light_entries, split_virtual_path,
    };

    #[test]
    fn rejects_parent_segments() {
        assert!(split_virtual_path("/repo/../secret").is_err());
        assert!(split_virtual_path("/repo\\secret").is_err());
    }

    #[tokio::test]
    async fn snapshot_resolves_mount_path() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!("web-file-browser-resolver-test-{nonce}"));
        fs::create_dir_all(&temp).unwrap();
        fs::write(temp.join("a.txt"), "hello").unwrap();

        let snapshot = MappingSnapshot::build(vec![PathMapping {
            id: Some(1),
            mount_path: "/repo".to_string(),
            folder_path: temp.to_string_lossy().to_string(),
            remark: Some(String::new()),
            order: Some(0),
            writable: true,
        }])
        .await
        .unwrap();

        let resolved = resolve_existing_sync(&snapshot, "/repo/a.txt").unwrap();
        assert!(resolved.real_path.ends_with("a.txt"));
        assert!(metadata_sync(&snapshot, "/repo/a.txt").is_ok());

        fs::remove_dir_all(temp).unwrap();
    }

    #[tokio::test]
    async fn no_follow_final_keeps_final_symlink_when_available() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp =
            std::env::temp_dir().join(format!("web-file-browser-resolver-symlink-test-{nonce}"));
        fs::create_dir_all(&temp).unwrap();
        let source = temp.join("source.txt");
        let link = temp.join("source-link.txt");
        fs::write(&source, "hello").unwrap();
        if !try_create_file_symlink(&source, &link) {
            fs::remove_dir_all(temp).unwrap();
            return;
        }

        let snapshot = MappingSnapshot::build(vec![PathMapping {
            id: Some(1),
            mount_path: "/repo".to_string(),
            folder_path: temp.to_string_lossy().to_string(),
            remark: Some(String::new()),
            order: Some(0),
            writable: true,
        }])
        .await
        .unwrap();

        let followed = resolve_existing_sync(&snapshot, "/repo/source-link.txt").unwrap();
        let not_followed =
            resolve_existing_no_follow_final_sync(&snapshot, "/repo/source-link.txt").unwrap();

        assert!(followed.real_path.ends_with("source.txt"));
        assert!(not_followed.real_path.ends_with("source-link.txt"));
        assert!(
            fs::symlink_metadata(&not_followed.real_path)
                .unwrap()
                .file_type()
                .is_symlink()
        );

        fs::remove_dir_all(temp).unwrap();
    }

    #[tokio::test]
    async fn empty_snapshot_only_serves_virtual_root() {
        let snapshot = MappingSnapshot::build(Vec::new()).await.unwrap();

        assert!(metadata_sync(&snapshot, "/").is_ok());
        assert!(metadata_sync(&snapshot, "/missing").is_err());
    }

    #[tokio::test]
    async fn basic_modified_precheck_uses_real_target_only() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!("web-file-browser-modified-test-{nonce}"));
        fs::create_dir_all(&temp).unwrap();
        fs::write(temp.join("a.txt"), "hello").unwrap();

        let snapshot = MappingSnapshot::build(vec![PathMapping {
            id: Some(1),
            mount_path: "/repo".to_string(),
            folder_path: temp.to_string_lossy().to_string(),
            remark: Some(String::new()),
            order: Some(0),
            writable: true,
        }])
        .await
        .unwrap();

        let root_modified = basic_metadata_modified_sync(&snapshot, "/").unwrap();
        let folder_modified = basic_metadata_modified_sync(&snapshot, "/repo").unwrap();
        let file_modified = basic_metadata_modified_sync(&snapshot, "/repo/a.txt").unwrap();

        assert_eq!(root_modified, None);
        assert!(
            folder_modified
                .as_deref()
                .is_some_and(|value| !value.is_empty())
        );
        assert!(
            file_modified
                .as_deref()
                .is_some_and(|value| !value.is_empty())
        );

        fs::remove_dir_all(temp).unwrap();
    }

    #[tokio::test]
    async fn paginates_folder_metadata() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!("web-file-browser-page-test-{nonce}"));
        fs::create_dir_all(temp.join("folder-a")).unwrap();
        fs::create_dir_all(temp.join("folder-b")).unwrap();
        fs::create_dir_all(temp.join("folder-c")).unwrap();
        fs::write(temp.join("a.txt"), "a").unwrap();
        fs::write(temp.join("b.txt"), "b").unwrap();
        fs::write(temp.join("c.txt"), "c").unwrap();

        let snapshot = MappingSnapshot::build(vec![PathMapping {
            id: Some(1),
            mount_path: "/repo".to_string(),
            folder_path: temp.to_string_lossy().to_string(),
            remark: Some(String::new()),
            order: Some(0),
            writable: true,
        }])
        .await
        .unwrap();

        let MetadataEntry::Folder { data, .. } = metadata_sync_with_options(
            &snapshot,
            "/repo",
            DirectoryListOptions {
                offset: 1,
                limit: Some(1),
                detail: DirectoryDetail::Full,
                include_total: true,
                ..DirectoryListOptions::default()
            },
        )
        .unwrap() else {
            panic!("expected folder metadata");
        };

        assert_eq!(data.folder_total, Some(3));
        assert_eq!(data.file_total, Some(3));
        assert_eq!(data.folder.len(), 1);
        assert_eq!(data.file.len(), 0);
        assert_eq!(data.folder[0].name, "folder-b");
        assert_eq!(data.has_more, Some(true));

        fs::remove_dir_all(temp).unwrap();
    }

    #[tokio::test]
    async fn omits_totals_by_default_but_keeps_has_more() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!("web-file-browser-total-test-{nonce}"));
        fs::create_dir_all(temp.join("folder-a")).unwrap();
        fs::create_dir_all(temp.join("folder-b")).unwrap();
        fs::create_dir_all(temp.join("folder-c")).unwrap();
        fs::write(temp.join("a.txt"), "a").unwrap();
        fs::write(temp.join("b.txt"), "b").unwrap();
        fs::write(temp.join("c.txt"), "c").unwrap();

        let snapshot = MappingSnapshot::build(vec![PathMapping {
            id: Some(1),
            mount_path: "/repo".to_string(),
            folder_path: temp.to_string_lossy().to_string(),
            remark: Some(String::new()),
            order: Some(0),
            writable: true,
        }])
        .await
        .unwrap();

        let MetadataEntry::Folder { data, .. } = metadata_sync_with_options(
            &snapshot,
            "/repo",
            DirectoryListOptions {
                offset: 2,
                limit: Some(2),
                detail: DirectoryDetail::Basic,
                ..DirectoryListOptions::default()
            },
        )
        .unwrap() else {
            panic!("expected folder metadata");
        };

        assert_eq!(data.folder_total, None);
        assert_eq!(data.file_total, None);
        assert_eq!(data.folder.len(), 1);
        assert_eq!(data.file.len(), 1);
        assert_eq!(data.has_more, Some(true));

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn paged_light_read_keeps_only_requested_sort_window() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!("web-file-browser-window-test-{nonce}"));
        fs::create_dir_all(&temp).unwrap();
        for index in 0..50 {
            fs::write(temp.join(format!("file-{index:02}.txt")), "x").unwrap();
        }

        let options = DirectoryListOptions {
            offset: 10,
            limit: Some(5),
            ..DirectoryListOptions::default()
        };
        let mut result = read_light_entries_page(&temp, options).unwrap();

        assert_eq!(result.total_entries, 50);
        assert_eq!(result.folder_total, 0);
        assert_eq!(result.file_total, 50);
        assert_eq!(result.entries.len(), 15);

        sort_light_entries(&mut result.entries, options.order);
        let names = page_entries(result.entries, options)
            .into_iter()
            .map(|entry| entry.name)
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            vec![
                "file-10.txt",
                "file-11.txt",
                "file-12.txt",
                "file-13.txt",
                "file-14.txt",
            ]
        );

        fs::remove_dir_all(temp).unwrap();
    }

    fn try_create_file_symlink(source: &std::path::Path, link: &std::path::Path) -> bool {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(source, link).is_ok()
        }
        #[cfg(windows)]
        {
            std::os::windows::fs::symlink_file(source, link).is_ok()
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = (source, link);
            false
        }
    }
}
