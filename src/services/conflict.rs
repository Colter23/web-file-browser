use std::{
    fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;

use crate::{error::AppError, models::ConflictPolicy};

#[derive(Debug, Clone)]
pub struct ConflictTarget {
    pub path: PathBuf,
    pub name: String,
    pub existed: bool,
}

pub fn resolve_child(
    parent: &Path,
    desired_name: &str,
    display_path: &str,
    policy: ConflictPolicy,
) -> Result<ConflictTarget, AppError> {
    let target = parent.join(desired_name);
    match policy {
        ConflictPolicy::Reject => {
            if target.exists() {
                Err(AppError::conflict(format!("路径已存在: {display_path}"))
                    .with_reason("PATH_ALREADY_EXISTS")
                    .with_param("path", display_path))
            } else {
                Ok(ConflictTarget {
                    path: target,
                    name: desired_name.to_string(),
                    existed: false,
                })
            }
        }
        ConflictPolicy::Overwrite => Ok(ConflictTarget {
            existed: target.exists(),
            path: target,
            name: desired_name.to_string(),
        }),
        ConflictPolicy::AutoRename => auto_rename_child(parent, desired_name),
    }
}

pub fn ensure_file_overwrite_allowed(target: &ConflictTarget) -> Result<(), AppError> {
    if target.existed && !target.path.is_file() {
        return Err(AppError::conflict("仅支持显式覆盖文件，不覆盖目录")
            .with_reason("OVERWRITE_DIR_FORBIDDEN"));
    }
    Ok(())
}

pub fn replace_file_sync(source: &Path, target: &Path) -> Result<(), AppError> {
    match fs::rename(source, target) {
        Ok(()) => Ok(()),
        Err(error) if target.exists() => replace_existing_file_sync(source, target, error),
        Err(rename_error) => move_file_with_copy_fallback(source, target).map_err(|copy_error| {
            AppError::internal(format!("移动文件失败: {rename_error}; {copy_error}"))
        }),
    }
}

fn replace_existing_file_sync(
    source: &Path,
    target: &Path,
    first_error: std::io::Error,
) -> Result<(), AppError> {
    if !target.is_file() {
        return Err(
            AppError::conflict("仅支持替换文件，不替换目录").with_reason("OVERWRITE_DIR_FORBIDDEN")
        );
    }

    let backup = backup_sibling_path(target);
    fs::rename(target, &backup).map_err(|backup_error| {
        AppError::internal(format!("准备替换文件失败: {first_error}; {backup_error}"))
    })?;

    match move_file_with_copy_fallback(source, target) {
        Ok(()) => {
            let _ = fs::remove_file(&backup);
            Ok(())
        }
        Err(move_error) => {
            let _ = fs::remove_file(target);
            let restore_result = fs::rename(&backup, target);
            match restore_result {
                Ok(()) => Err(AppError::internal(format!(
                    "替换文件失败，已恢复旧文件: {first_error}; {move_error}"
                ))),
                Err(restore_error) => Err(AppError::internal(format!(
                    "替换文件失败，且恢复旧文件失败: {first_error}; {move_error}; {restore_error}"
                ))),
            }
        }
    }
}

fn move_file_with_copy_fallback(source: &Path, target: &Path) -> Result<(), std::io::Error> {
    match fs::rename(source, target) {
        Ok(()) => Ok(()),
        Err(rename_error) => fs::copy(source, target)
            .and_then(|_| fs::remove_file(source))
            .map_err(|copy_error| {
                std::io::Error::new(copy_error.kind(), format!("{rename_error}; {copy_error}"))
            }),
    }
}

fn backup_sibling_path(target: &Path) -> PathBuf {
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("target");
    parent.join(format!(".{name}.replace-backup-{}", Uuid::new_v4()))
}

fn auto_rename_child(parent: &Path, desired_name: &str) -> Result<ConflictTarget, AppError> {
    let target = parent.join(desired_name);
    if !target.exists() {
        return Ok(ConflictTarget {
            path: target,
            name: desired_name.to_string(),
            existed: false,
        });
    }

    for index in 1..10_000 {
        let candidate_name = numbered_name(desired_name, index);
        let candidate = parent.join(&candidate_name);
        if !candidate.exists() {
            return Ok(ConflictTarget {
                path: candidate,
                name: candidate_name,
                existed: false,
            });
        }
    }

    Err(AppError::conflict("无法生成不冲突的文件名").with_reason("AUTO_RENAME_EXHAUSTED"))
}

fn numbered_name(name: &str, index: usize) -> String {
    let path = Path::new(name);
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return format!("{name} ({index})");
    };
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(name);
    format!("{stem} ({index}).{extension}")
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::models::ConflictPolicy;

    use super::{backup_sibling_path, replace_file_sync, resolve_child};

    #[test]
    fn auto_renames_existing_file_with_extension() {
        let temp = temp_dir("web-file-browser-conflict-test");
        fs::create_dir_all(&temp).unwrap();
        fs::write(temp.join("a.txt"), "old").unwrap();

        let target =
            resolve_child(&temp, "a.txt", "/repo/a.txt", ConflictPolicy::AutoRename).unwrap();

        assert_eq!(target.name, "a (1).txt");
        assert_eq!(target.path, temp.join("a (1).txt"));
        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn reject_fails_when_target_exists() {
        let temp = temp_dir("web-file-browser-conflict-reject-test");
        fs::create_dir_all(&temp).unwrap();
        fs::write(temp.join("a.txt"), "old").unwrap();

        assert!(resolve_child(&temp, "a.txt", "/repo/a.txt", ConflictPolicy::Reject).is_err());
        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn replace_file_sync_overwrites_existing_file() {
        let temp = temp_dir("web-file-browser-conflict-replace-test");
        fs::create_dir_all(&temp).unwrap();
        let source = temp.join("source.txt");
        let target = temp.join("target.txt");
        fs::write(&source, "new").unwrap();
        fs::write(&target, "old").unwrap();

        replace_file_sync(&source, &target).unwrap();

        assert_eq!(fs::read_to_string(target).unwrap(), "new");
        assert!(!source.exists());
        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn replace_file_sync_rejects_directory_target() {
        let temp = temp_dir("web-file-browser-conflict-replace-dir-test");
        fs::create_dir_all(&temp).unwrap();
        let source = temp.join("source.txt");
        let target = temp.join("target");
        fs::write(&source, "new").unwrap();
        fs::create_dir(&target).unwrap();

        assert!(replace_file_sync(&source, &target).is_err());
        assert!(source.exists());
        assert!(target.is_dir());
        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn backup_path_stays_next_to_target() {
        let temp = temp_dir("web-file-browser-conflict-backup-path-test");
        fs::create_dir_all(&temp).unwrap();
        let target = temp.join("target.txt");

        let backup = backup_sibling_path(&target);

        assert_eq!(backup.parent(), Some(temp.as_path()));
        assert!(
            backup
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap()
                .starts_with(".target.txt.replace-backup-")
        );
        fs::remove_dir_all(temp).unwrap();
    }

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nonce}"))
    }
}
