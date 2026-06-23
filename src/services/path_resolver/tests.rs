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
    let temp = std::env::temp_dir().join(format!("web-file-browser-resolver-symlink-test-{nonce}"));
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
