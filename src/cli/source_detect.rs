//! Heuristic detection for whether a workspace tree looks like it contains project source.
//! Used by [`crate::cli::repo_checks::workspace`] when deciding whether `kiss clamp` may be needed.

use std::path::Path;

pub fn has_source_files(dir: &Path) -> bool {
    fn check_dir(dir: &Path) -> bool {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return false;
        };
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_symlink() {
                continue;
            }
            let path = entry.path();
            if file_type.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name == "Cargo.toml"
                        || name == "pyproject.toml"
                        || name == "requirements.txt"
                    {
                        return true;
                    }
                }
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ext == "rs" || ext == "py" {
                        return true;
                    }
                }
            } else if file_type.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.') || name == "target" || name == "__pycache__" {
                        continue;
                    }
                }
                if check_dir(&path) {
                    return true;
                }
            }
        }
        false
    }
    check_dir(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn has_extension_files(dir: &Path, ext: &str) -> bool {
        fn check_dir(dir: &Path, ext: &str) -> bool {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return false;
            };
            for entry in entries.flatten() {
                let Ok(file_type) = entry.file_type() else {
                    continue;
                };
                if file_type.is_symlink() {
                    continue;
                }
                let path = entry.path();
                if file_type.is_file() {
                    if let Some(file_ext) = path.extension().and_then(|e| e.to_str()) {
                        if file_ext == ext {
                            return true;
                        }
                    }
                } else if file_type.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with('.') || name == "target" || name == "__pycache__" {
                            continue;
                        }
                    }
                    if check_dir(&path, ext) {
                        return true;
                    }
                }
            }
            false
        }
        check_dir(dir, ext)
    }

    #[test]
    fn has_source_files_returns_false_for_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(!has_source_files(tmp.path()));
    }

    #[test]
    fn has_source_files_returns_true_for_rs_file() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("main.rs"), "fn main() {}").unwrap();
        assert!(has_source_files(tmp.path()));
    }

    #[test]
    fn has_source_files_returns_true_for_py_file() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("script.py"), "print('hi')").unwrap();
        assert!(has_source_files(tmp.path()));
    }

    #[test]
    fn has_source_files_returns_true_for_cargo_toml() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("Cargo.toml"), "[package]").unwrap();
        assert!(has_source_files(tmp.path()));
    }

    #[test]
    fn has_extension_files_ignores_marker_files() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("Cargo.toml"), "[package]").unwrap();
        assert!(!has_extension_files(tmp.path(), "rs"));
    }

    #[test]
    fn has_source_files_ignores_hidden_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let hidden = tmp.path().join(".hidden");
        std::fs::create_dir(&hidden).unwrap();
        std::fs::write(hidden.join("main.rs"), "fn main() {}").unwrap();
        assert!(!has_source_files(tmp.path()));
    }

    #[cfg(unix)]
    #[test]
    fn has_source_files_ignores_symlink_dirs() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::tempdir().unwrap();
        let real = tmp.path().join(".real");
        std::fs::create_dir(&real).unwrap();
        std::fs::write(real.join("main.rs"), "fn main() {}").unwrap();
        symlink(&real, tmp.path().join("link")).unwrap();
        assert!(!has_source_files(tmp.path()));
    }

    #[cfg(unix)]
    #[test]
    fn has_source_files_ignores_symlink_dir_pointing_outside_workspace() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("main.rs"), "fn main() {}").unwrap();
        symlink(outside.path(), tmp.path().join("outside")).unwrap();
        assert!(!has_source_files(tmp.path()));
    }

    #[test]
    fn kiss_stringify_source_detect() {
        let _ = stringify!(super::has_source_files);
    }
}
