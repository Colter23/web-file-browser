use std::{fs, sync::Arc};

use crate::{
    error::AppError,
    models::{
        ConflictPolicy, CreateEntryRequest, CreateEntryType, FileOperationResponse,
        MoveEntryRequest,
    },
    services::{
        conflict,
        path_resolver::{
            MappingSnapshot, ResolvedPath, ensure_folder, ensure_not_mount_root, ensure_writable,
            join_virtual_path, normalize_child_name, resolve_existing_no_follow_final_sync,
            resolve_existing_sync, split_virtual_path, virtual_path_from_parts,
        },
    },
};

pub async fn create_entry(
    snapshot: Arc<MappingSnapshot>,
    parent_path: String,
    request: CreateEntryRequest,
    policy: ConflictPolicy,
) -> Result<FileOperationResponse, AppError> {
    tokio::task::spawn_blocking(move || create_entry_sync(&snapshot, &parent_path, request, policy))
        .await?
}

pub async fn move_entry(
    snapshot: Arc<MappingSnapshot>,
    source_path: String,
    request: MoveEntryRequest,
    policy: ConflictPolicy,
) -> Result<FileOperationResponse, AppError> {
    tokio::task::spawn_blocking(move || move_entry_sync(&snapshot, &source_path, &request, policy))
        .await?
}

pub async fn resolve_delete_target(
    snapshot: Arc<MappingSnapshot>,
    path: String,
) -> Result<ResolvedPath, AppError> {
    tokio::task::spawn_blocking(move || {
        let resolved = resolve_existing_no_follow_final_sync(&snapshot, &path)?;
        ensure_writable(&resolved.mapping)?;
        ensure_not_mount_root(&resolved)?;
        ensure_no_symlink(&resolved.real_path, "不支持删除符号链接")?;
        Ok(resolved)
    })
    .await?
}

fn create_entry_sync(
    snapshot: &MappingSnapshot,
    parent_path: &str,
    request: CreateEntryRequest,
    policy: ConflictPolicy,
) -> Result<FileOperationResponse, AppError> {
    let parent = resolve_existing_sync(snapshot, parent_path)?;
    ensure_writable(&parent.mapping)?;
    ensure_folder(&parent.real_path, &parent.virtual_path)?;
    let name = normalize_child_name(&request.name)?;
    let virtual_path = join_virtual_path(&parent.virtual_path, &name);
    let target = conflict::resolve_child(&parent.real_path, &name, &virtual_path, policy)?;

    match request.entry_type {
        CreateEntryType::File => {
            conflict::ensure_file_overwrite_allowed(&target)?;
            fs::write(&target.path, [])?;
        }
        CreateEntryType::Folder => {
            if target.existed {
                return Err(AppError::conflict("仅支持显式覆盖文件，不覆盖目录"));
            }
            fs::create_dir(&target.path)?;
        }
    }

    Ok(FileOperationResponse {
        path: join_virtual_path(&parent.virtual_path, &target.name),
    })
}

fn move_entry_sync(
    snapshot: &MappingSnapshot,
    source_path: &str,
    request: &MoveEntryRequest,
    policy: ConflictPolicy,
) -> Result<FileOperationResponse, AppError> {
    let source = resolve_existing_no_follow_final_sync(snapshot, source_path)?;
    ensure_writable(&source.mapping)?;
    ensure_not_mount_root(&source)?;
    ensure_no_symlink(&source.real_path, "不支持移动符号链接")?;

    let target_parts = split_virtual_path(&request.target_path)?;
    let Some(target_name) = target_parts.last() else {
        return Err(AppError::bad_request("目标路径不能为空"));
    };
    let target_name = normalize_child_name(target_name)?;
    let target_parent_parts = target_parts[..target_parts.len() - 1].to_vec();
    let target_parent_path = virtual_path_from_parts(&target_parent_parts);
    let target_parent = resolve_existing_sync(snapshot, &target_parent_path)?;
    ensure_writable(&target_parent.mapping)?;
    ensure_folder(&target_parent.real_path, &target_parent.virtual_path)?;

    if source.mapping.id != target_parent.mapping.id
        || source.mapping.mount_path != target_parent.mapping.mount_path
    {
        return Err(AppError::bad_request("不能跨挂载点移动文件"));
    }

    let desired_virtual_path = join_virtual_path(&target_parent.virtual_path, &target_name);
    let desired_target = target_parent.real_path.join(&target_name);
    if desired_target.exists()
        && let Ok(canonical_target) = desired_target.canonicalize()
        && canonical_target == source.real_path
    {
        return Ok(FileOperationResponse {
            path: desired_virtual_path,
        });
    }

    let target = conflict::resolve_child(
        &target_parent.real_path,
        &target_name,
        &desired_virtual_path,
        policy,
    )?;

    if source.real_path.is_dir() && target.path.starts_with(&source.real_path) {
        return Err(AppError::bad_request("不能把文件夹移动到自身内部"));
    }

    if target.existed {
        if source.real_path.is_dir() {
            return Err(AppError::conflict("不支持覆盖移动目录"));
        }
        conflict::ensure_file_overwrite_allowed(&target)?;
        conflict::replace_file_sync(&source.real_path, &target.path)?;
    } else {
        fs::rename(&source.real_path, &target.path)?;
    }
    Ok(FileOperationResponse {
        path: join_virtual_path(&target_parent.virtual_path, &target.name),
    })
}

fn ensure_no_symlink(path: &std::path::Path, message: &str) -> Result<(), AppError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err(AppError::bad_request(format!(
            "{message}: {}",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::models::{
        ConflictPolicy, CreateEntryRequest, CreateEntryType, MoveEntryRequest, PathMapping,
    };

    use super::{create_entry_sync, move_entry_sync};
    use crate::services::path_resolver::MappingSnapshot;

    #[tokio::test]
    async fn creates_and_moves_inside_mount() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!("web-file-browser-file-ops-test-{nonce}"));
        fs::create_dir_all(&temp).unwrap();
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

        create_entry_sync(
            &snapshot,
            "/repo",
            CreateEntryRequest {
                entry_type: CreateEntryType::File,
                name: "a.txt".to_string(),
                conflict_policy: None,
            },
            ConflictPolicy::Reject,
        )
        .unwrap();
        move_entry_sync(
            &snapshot,
            "/repo/a.txt",
            &MoveEntryRequest {
                target_path: "/repo/b.txt".to_string(),
                conflict_policy: None,
            },
            ConflictPolicy::Reject,
        )
        .unwrap();

        assert!(temp.join("b.txt").is_file());
        fs::remove_dir_all(temp).unwrap();
    }

    #[tokio::test]
    async fn rejects_writes_on_readonly_mount() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!("web-file-browser-readonly-test-{nonce}"));
        fs::create_dir_all(&temp).unwrap();
        let snapshot = MappingSnapshot::build(vec![PathMapping {
            id: Some(1),
            mount_path: "/repo".to_string(),
            folder_path: temp.to_string_lossy().to_string(),
            remark: Some(String::new()),
            order: Some(0),
            writable: false,
        }])
        .await
        .unwrap();

        let result = create_entry_sync(
            &snapshot,
            "/repo",
            CreateEntryRequest {
                entry_type: CreateEntryType::File,
                name: "a.txt".to_string(),
                conflict_policy: None,
            },
            ConflictPolicy::Reject,
        );

        assert!(result.is_err());
        assert!(!temp.join("a.txt").exists());
        fs::remove_dir_all(temp).unwrap();
    }

    #[tokio::test]
    async fn rejects_cross_mount_move() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let left = std::env::temp_dir().join(format!("web-file-browser-left-test-{nonce}"));
        let right = std::env::temp_dir().join(format!("web-file-browser-right-test-{nonce}"));
        fs::create_dir_all(&left).unwrap();
        fs::create_dir_all(&right).unwrap();
        fs::write(left.join("a.txt"), "hello").unwrap();
        let snapshot = MappingSnapshot::build(vec![
            PathMapping {
                id: Some(1),
                mount_path: "/left".to_string(),
                folder_path: left.to_string_lossy().to_string(),
                remark: Some(String::new()),
                order: Some(0),
                writable: true,
            },
            PathMapping {
                id: Some(2),
                mount_path: "/right".to_string(),
                folder_path: right.to_string_lossy().to_string(),
                remark: Some(String::new()),
                order: Some(1),
                writable: true,
            },
        ])
        .await
        .unwrap();

        let result = move_entry_sync(
            &snapshot,
            "/left/a.txt",
            &MoveEntryRequest {
                target_path: "/right/a.txt".to_string(),
                conflict_policy: None,
            },
            ConflictPolicy::Reject,
        );

        assert!(result.is_err());
        assert!(left.join("a.txt").is_file());
        assert!(!right.join("a.txt").exists());
        fs::remove_dir_all(left).unwrap();
        fs::remove_dir_all(right).unwrap();
    }

    #[tokio::test]
    async fn auto_renames_create_and_move_conflicts() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = std::env::temp_dir().join(format!("web-file-browser-rename-test-{nonce}"));
        fs::create_dir_all(&temp).unwrap();
        fs::write(temp.join("a.txt"), "old").unwrap();
        fs::write(temp.join("source.txt"), "source").unwrap();
        fs::write(temp.join("target.txt"), "target").unwrap();
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

        let created = create_entry_sync(
            &snapshot,
            "/repo",
            CreateEntryRequest {
                entry_type: CreateEntryType::File,
                name: "a.txt".to_string(),
                conflict_policy: None,
            },
            ConflictPolicy::AutoRename,
        )
        .unwrap();
        let moved = move_entry_sync(
            &snapshot,
            "/repo/source.txt",
            &MoveEntryRequest {
                target_path: "/repo/target.txt".to_string(),
                conflict_policy: None,
            },
            ConflictPolicy::AutoRename,
        )
        .unwrap();

        assert_eq!(created.path, "/repo/a (1).txt");
        assert_eq!(moved.path, "/repo/target (1).txt");
        assert!(temp.join("a (1).txt").is_file());
        assert!(temp.join("target (1).txt").is_file());
        fs::remove_dir_all(temp).unwrap();
    }
}
