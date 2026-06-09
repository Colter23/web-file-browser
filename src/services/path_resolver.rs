use std::{
    cmp::Reverse,
    fs,
    path::{Component, Path, PathBuf},
    sync::Arc,
    time::UNIX_EPOCH,
};

use crate::{
    error::AppError,
    models::{EntryKind, FileInfo, FolderData, FolderInfo, FolderNode, PathMapping},
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

pub enum MetadataEntry {
    Folder(FolderData),
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

pub async fn resolve_existing(
    snapshot: Arc<MappingSnapshot>,
    path: String,
) -> Result<ResolvedPath, AppError> {
    tokio::task::spawn_blocking(move || resolve_existing_sync(&snapshot, &path)).await?
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
            Ok(MetadataEntry::Folder(list_real_folder(
                &real_path,
                &parent_path,
                options,
            )?))
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
            return Ok(MetadataEntry::Folder(FolderData::full(
                virtual_path,
                Vec::new(),
                Vec::new(),
            )));
        };
        let Some(FolderNode::Virtual { children, .. }) = resolve_virtual_node(root, &parts) else {
            return Err(AppError::not_found(format!("查无此路径: {virtual_path}")));
        };

        Ok(MetadataEntry::Folder(list_virtual_folder(
            children,
            virtual_path,
            options,
        )))
    }
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
    path: PathBuf,
    kind: EntryKind,
}

#[derive(Debug)]
struct DetailedEntry {
    light: LightEntry,
    metadata: fs::Metadata,
}

fn read_light_entries(
    path: &Path,
    options: DirectoryListOptions,
) -> Result<Vec<LightEntry>, AppError> {
    let mut entries = Vec::new();
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
        entries.push(LightEntry {
            name,
            path: entry.path(),
            kind,
        });
    }
    Ok(entries)
}

fn list_real_folder_basic(
    path: &Path,
    parent_path: &str,
    options: DirectoryListOptions,
) -> Result<FolderData, AppError> {
    let mut entries = read_light_entries(path, options)?;
    sort_light_entries(&mut entries, options.order);
    let (folder_total, file_total) = count_kinds(entries.iter().map(|entry| entry.kind));
    let entries = page_entries(entries, options);
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
        folder_total,
        file_total,
        options,
    ))
}

fn list_real_folder_full(
    path: &Path,
    parent_path: &str,
    options: DirectoryListOptions,
) -> Result<FolderData, AppError> {
    let entries = read_light_entries(path, options)?;
    let (folder_total, file_total) = count_kinds(entries.iter().map(|entry| entry.kind));
    let mut entries = entries
        .into_iter()
        .map(|entry| {
            let metadata = fs::metadata(&entry.path)?;
            Ok(DetailedEntry {
                light: entry,
                metadata,
            })
        })
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    sort_detailed_entries(&mut entries, options.sort, options.order);
    let entries = page_entries(entries, options);
    let mut folder = Vec::new();
    let mut file = Vec::new();
    for entry in entries {
        match entry.light.kind {
            EntryKind::Folder => folder.push(folder_info_from_detailed_entry(&entry, parent_path)),
            EntryKind::File => file.push(file_info_from_detailed_entry(&entry, parent_path)),
        }
    }

    Ok(folder_data(
        parent_path.to_string(),
        folder,
        file,
        folder_total,
        file_total,
        options,
    ))
}

fn folder_data(
    path: String,
    folder: Vec<FolderInfo>,
    file: Vec<FileInfo>,
    folder_total: usize,
    file_total: usize,
    options: DirectoryListOptions,
) -> FolderData {
    if let Some(limit) = options.limit {
        FolderData::paged(
            path,
            folder,
            file,
            folder_total,
            file_total,
            options.offset,
            limit,
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
        let folder_total = folder.len();
        let folder = folder
            .into_iter()
            .skip(options.offset)
            .take(limit)
            .collect::<Vec<_>>();
        FolderData::paged(
            virtual_path,
            folder,
            Vec::new(),
            folder_total,
            0,
            options.offset,
            limit,
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
    let extension = entry
        .light
        .path
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
    entries.sort_by(|left, right| {
        kind_order(left.kind)
            .cmp(&kind_order(right.kind))
            .then_with(|| left.name.cmp(&right.name))
    });
    if order == SortOrder::Desc {
        entries.reverse();
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

fn count_kinds(kinds: impl Iterator<Item = EntryKind>) -> (usize, usize) {
    let mut folder_total = 0;
    let mut file_total = 0;
    for kind in kinds {
        match kind {
            EntryKind::Folder => folder_total += 1,
            EntryKind::File => file_total += 1,
        }
    }
    (folder_total, file_total)
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
        DirectoryDetail, DirectoryListOptions, MappingSnapshot, MetadataEntry, metadata_sync,
        metadata_sync_with_options, resolve_existing_sync, split_virtual_path,
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
    async fn empty_snapshot_only_serves_virtual_root() {
        let snapshot = MappingSnapshot::build(Vec::new()).await.unwrap();

        assert!(metadata_sync(&snapshot, "/").is_ok());
        assert!(metadata_sync(&snapshot, "/missing").is_err());
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

        let MetadataEntry::Folder(data) = metadata_sync_with_options(
            &snapshot,
            "/repo",
            DirectoryListOptions {
                offset: 1,
                limit: Some(1),
                detail: DirectoryDetail::Full,
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
}
