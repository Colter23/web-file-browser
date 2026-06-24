use std::{cmp::Ordering, collections::BinaryHeap, fs, path::Path, time::UNIX_EPOCH};

use crate::{
    error::AppError,
    models::{EntryKind, FileInfo, FolderData, FolderInfo, FolderNode, FolderPageInfo},
    services::reserved,
};

use super::{
    DirectoryDetail, DirectoryListOptions, DirectorySort, EntryFilter, SortOrder, join_virtual_path,
};

pub(super) fn list_real_folder(
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
        if reserved::is_mount_trash_dir_name(&name)
            || (!options.include_hidden && name.starts_with('.'))
        {
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

pub(super) fn list_virtual_folder(
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

pub(super) fn file_info_from_path(
    path: &Path,
    virtual_path: &str,
    metadata: &fs::Metadata,
) -> FileInfo {
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

pub(super) fn modified_to_string(metadata: &fs::Metadata) -> String {
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

    use super::{page_entries, read_light_entries_page, sort_light_entries};
    use crate::services::path_resolver::DirectoryListOptions;

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
}
