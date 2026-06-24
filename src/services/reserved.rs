use std::path::{Path, PathBuf};

pub const MOUNT_TRASH_DIR_NAME: &str = ".web-file-browser-trash";

pub fn is_mount_trash_dir_name(name: &str) -> bool {
    name == MOUNT_TRASH_DIR_NAME
}

pub fn mount_trash_root(mount_root: &Path) -> PathBuf {
    mount_root.join(MOUNT_TRASH_DIR_NAME)
}
