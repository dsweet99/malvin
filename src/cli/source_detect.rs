//! Heuristic detection for whether a workspace tree looks like it contains project source.
//! Used by [`crate::repo_checks::gate_run`] when deciding whether `kiss clamp` may be needed.

use std::path::{Path, PathBuf};

fn entry_name_has_extension(path: &Path, ext: &str) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some(ext)
}

fn entry_name_is_workspace_marker(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|name| {
            name == "Cargo.toml" || name == "pyproject.toml" || name == "requirements.txt"
        })
}

fn resolved_symlink_target(link: &Path) -> Option<PathBuf> {
    let target = std::fs::read_link(link).ok()?;
    Some(if target.is_absolute() {
        target
    } else {
        link.parent()?.join(target)
    })
}

fn symlink_resolves_to_existing_file(link: &Path) -> bool {
    std::fs::metadata(link).is_ok_and(|m| m.is_file())
}

fn entry_or_symlink_file_target_matches(link: &Path, matches: impl Fn(&Path) -> bool) -> bool {
    if symlink_resolves_to_existing_file(link) && matches(link) {
        return true;
    }
    let Some(target) = resolved_symlink_target(link) else {
        return false;
    };
    target.is_file() && matches(&target)
}

pub fn has_extension_files(dir: &Path, ext: &str) -> bool {
    fn check_dir(dir: &Path, ext: &str) -> bool {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return false;
        };
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            let path = entry.path();
            if file_type.is_symlink() {
                if entry_or_symlink_file_target_matches(&path, |p| entry_name_has_extension(p, ext))
                {
                    return true;
                }
                continue;
            }
            if file_type.is_file() {
                if entry_name_has_extension(&path, ext) {
                    return true;
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

pub fn has_workspace_marker_files(dir: &Path) -> bool {
    fn check_dir(dir: &Path) -> bool {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return false;
        };
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            let path = entry.path();
            if file_type.is_symlink() {
                if entry_or_symlink_file_target_matches(&path, entry_name_is_workspace_marker) {
                    return true;
                }
                continue;
            }
            if file_type.is_file() {
                if entry_name_is_workspace_marker(&path) {
                    return true;
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

pub fn has_source_files(dir: &Path) -> bool {
    has_extension_files(dir, "rs")
        || has_extension_files(dir, "py")
        || has_workspace_marker_files(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn has_workspace_marker_files_returns_true_for_cargo_toml() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("Cargo.toml"), "[package]").unwrap();
        assert!(has_workspace_marker_files(tmp.path()));
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
    mod unix_symlink {
        use super::*;
        use std::os::unix::fs::symlink;

        #[test]
        fn has_source_files_ignores_symlink_dirs() {
            let tmp = tempfile::tempdir().unwrap();
            let real = tmp.path().join(".real");
            std::fs::create_dir(&real).unwrap();
            std::fs::write(real.join("main.rs"), "fn main() {}").unwrap();
            symlink(&real, tmp.path().join("link")).unwrap();
            assert!(!has_source_files(tmp.path()));
        }

        #[test]
        fn has_source_files_ignores_symlink_dir_pointing_outside_workspace() {
            let tmp = tempfile::tempdir().unwrap();
            let outside = tempfile::tempdir().unwrap();
            std::fs::write(outside.path().join("main.rs"), "fn main() {}").unwrap();
            symlink(outside.path(), tmp.path().join("outside")).unwrap();
            assert!(!has_source_files(tmp.path()));
        }

        #[test]
        fn has_source_files_detects_symlink_to_rs_file_in_workspace() {
            let tmp = tempfile::tempdir().unwrap();
            let outside = tempfile::tempdir().unwrap();
            let real = outside.path().join("real.rs");
            std::fs::write(&real, "fn main() {}").unwrap();
            symlink(&real, tmp.path().join("linked.rs")).unwrap();
            assert!(has_source_files(tmp.path()));
        }

        #[test]
        fn has_source_files_detects_symlink_to_cargo_toml_by_target_name() {
            let tmp = tempfile::tempdir().unwrap();
            let outside = tempfile::tempdir().unwrap();
            std::fs::write(outside.path().join("Cargo.toml"), "[package]").unwrap();
            symlink(
                outside.path().join("Cargo.toml"),
                tmp.path().join("manifest.toml"),
            )
            .unwrap();
            assert!(has_source_files(tmp.path()));
        }

        #[test]
        fn has_source_files_detects_symlink_to_rs_by_target_extension() {
            let tmp = tempfile::tempdir().unwrap();
            let outside = tempfile::tempdir().unwrap();
            let real = outside.path().join("real.rs");
            std::fs::write(&real, "fn main() {}").unwrap();
            symlink(&real, tmp.path().join("link")).unwrap();
            assert!(has_source_files(tmp.path()));
        }

        #[test]
        fn has_source_files_ignores_dangling_symlink_with_rs_link_name() {
            let tmp = tempfile::tempdir().unwrap();
            symlink(
                tmp.path().join("no_such_dir").join("missing.rs"),
                tmp.path().join("linked.rs"),
            )
            .unwrap();
            assert!(!has_source_files(tmp.path()));
        }
    }
}
