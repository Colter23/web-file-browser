use std::path::{Path, PathBuf};

pub const MOUNT_TRASH_DIR_NAME: &str = ".web-file-browser-trash";

pub fn is_mount_trash_dir_name(name: &str) -> bool {
    name.eq_ignore_ascii_case(MOUNT_TRASH_DIR_NAME)
}

pub fn mount_trash_root(mount_root: &Path) -> PathBuf {
    mount_root.join(MOUNT_TRASH_DIR_NAME)
}

#[cfg(test)]
mod tests {
    use super::is_mount_trash_dir_name;

    #[test]
    fn mount_trash_name_matches_case_insensitively() {
        assert!(is_mount_trash_dir_name(".web-file-browser-trash"));
        assert!(is_mount_trash_dir_name(".WEB-FILE-BROWSER-TRASH"));
        assert!(!is_mount_trash_dir_name(".web-file-browser-trash-backup"));
    }
}
