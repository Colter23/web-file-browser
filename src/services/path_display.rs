use std::path::Path;

pub fn display_path(path: &Path) -> String {
    let text = path.to_string_lossy();
    display_path_text(text.as_ref())
}

pub fn display_path_text(path: &str) -> String {
    strip_windows_extended_prefix(path)
}

pub fn has_windows_extended_prefix(path: &str) -> bool {
    path.starts_with(r"\\?\")
}

fn strip_windows_extended_prefix(path: &str) -> String {
    if let Some(rest) = path.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{rest}")
    } else if let Some(rest) = path.strip_prefix(r"\\?\") {
        rest.to_string()
    } else {
        path.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::strip_windows_extended_prefix;

    #[test]
    fn strips_windows_extended_drive_prefix() {
        assert_eq!(
            strip_windows_extended_prefix(r"\\?\C:\Downloads\Demo"),
            r"C:\Downloads\Demo"
        );
    }

    #[test]
    fn strips_windows_extended_unc_prefix() {
        assert_eq!(
            strip_windows_extended_prefix(r"\\?\UNC\server\share\Demo"),
            r"\\server\share\Demo"
        );
    }

    #[test]
    fn keeps_normal_path() {
        assert_eq!(
            strip_windows_extended_prefix(r"C:\Downloads\Demo"),
            r"C:\Downloads\Demo"
        );
        assert_eq!(strip_windows_extended_prefix("/mnt/files"), "/mnt/files");
    }
}
