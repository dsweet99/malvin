use std::path::Path;

fn root_has_python_project_markers(root: &Path) -> bool {
    if root.join("Cargo.toml").is_file() {
        return true;
    }
    for marker in [
        "pyproject.toml",
        "setup.py",
        "setup.cfg",
        "requirements.txt",
        "Pipfile",
        "poetry.lock",
    ] {
        if root.join(marker).is_file() {
            return true;
        }
    }
    root.join("tests").is_dir() || root_level_has_py_file(root)
}

/// Avoid recursively scanning arbitrary cwd trees (e.g. `$HOME`) when inferring Python gates.
fn should_walk_for_python_sources(root: &Path) -> bool {
    crate::repo_gates::git_worktree_toplevel(root).is_some()
}

fn root_level_has_py_file(root: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(root) else {
        return false;
    };
    entries.flatten().any(|entry| {
        entry
            .path()
            .extension()
            .and_then(|ext| ext.to_str())
            == Some("py")
    })
}

pub(super) fn visit_source_files(root: &Path, f: &mut impl FnMut(&Path)) {
    fn walk(dir: &Path, f: &mut impl FnMut(&Path)) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            let path = entry.path();
            if file_type.is_symlink() {
                if let Ok(md) = std::fs::metadata(&path) {
                    if md.is_file() {
                        f(&path);
                    }
                }
                continue;
            }
            if file_type.is_file() {
                f(&path);
            } else if file_type.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.') || name == "target" || name == "__pycache__" {
                        continue;
                    }
                }
                walk(&path, f);
            }
        }
    }
    walk(root, f);
}

fn root_level_python_flags(root: &Path) -> (bool, bool) {
    if !root_has_python_project_markers(root) {
        return (false, false);
    }
    let Ok(entries) = std::fs::read_dir(root) else {
        return (false, false);
    };
    let mut has_py = false;
    let mut has_pytest = false;
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        let is_py_file = if file_type.is_symlink() {
            std::fs::metadata(&path).ok().is_some_and(|md| md.is_file())
        } else {
            file_type.is_file()
        };
        if !is_py_file {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("py") {
            continue;
        }
        has_py = true;
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if stem.starts_with("test_") || stem.ends_with("_test") {
            has_pytest = true;
        }
    }
    (has_py, has_pytest)
}

pub(super) fn python_ruff_and_pytest_flags(root: &Path) -> (bool, bool) {
    if should_walk_for_python_sources(root) {
        let mut has_py = false;
        let mut has_pytest = false;
        visit_source_files(root, &mut |path: &Path| {
            if path.extension().and_then(|e| e.to_str()) != Some("py") {
                return;
            }
            has_py = true;
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                return;
            };
            if stem.starts_with("test_") || stem.ends_with("_test") {
                has_pytest = true;
            }
        });
        (has_py, has_pytest)
    } else {
        root_level_python_flags(root)
    }
}

#[cfg(test)]
mod tests {
    use super::python_ruff_and_pytest_flags;

    #[test]
    fn visit_source_files_empty_dir_has_no_python_flags() {
        let tmp = tempfile::tempdir().unwrap();
        let mut count = 0usize;
        super::visit_source_files(tmp.path(), &mut |_p| count += 1);
        assert_eq!(count, 0);
        let (has_py, has_pytest) = python_ruff_and_pytest_flags(tmp.path());
        assert!(!has_py);
        assert!(!has_pytest);
    }

    #[test]
    fn python_ruff_and_pytest_flags_skips_nested_py_without_workspace_markers() {
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("pkg");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("test_foo.py"), "def test_x():\n    assert True\n").unwrap();
        let (has_py, has_pytest) = python_ruff_and_pytest_flags(tmp.path());
        assert!(!has_py);
        assert!(!has_pytest);
    }

    #[test]
    fn python_ruff_and_pytest_flags_skips_malvin_dir_only_with_nested_py() {
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("pkg");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("main.py"), "x = 1\n").unwrap();
        std::fs::create_dir_all(tmp.path().join(".malvin")).unwrap();
        let (has_py, has_pytest) = python_ruff_and_pytest_flags(tmp.path());
        assert!(!has_py);
        assert!(!has_pytest);
    }

    #[test]
    fn python_ruff_and_pytest_flags_skips_nested_py_when_only_malvin_checks_present() {
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("pkg");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("test_foo.py"), "def test_x():\n    assert True\n").unwrap();
        std::fs::create_dir_all(tmp.path().join(".malvin")).unwrap();
        std::fs::write(tmp.path().join(".malvin/checks"), "kiss check\n").unwrap();
        let (has_py, has_pytest) = python_ruff_and_pytest_flags(tmp.path());
        assert!(!has_py);
        assert!(!has_pytest);
    }

    #[test]
    fn python_ruff_and_pytest_flags_finds_nested_py_in_git_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let w = tmp.path();
        std::fs::create_dir_all(w.join("pkg")).unwrap();
        std::fs::write(w.join("pkg/test_foo.py"), "def test_x():\n    assert True\n").unwrap();
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(w)
            .status()
            .expect("git init");
        let (has_py, has_pytest) = python_ruff_and_pytest_flags(w);
        assert!(has_py);
        assert!(has_pytest);
    }

    #[cfg(unix)]
    #[test]
    fn python_flags_see_symlinked_py_file() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::tempdir().unwrap();
        let w = tmp.path();
        std::fs::write(w.join("real.py"), "x = 1\n").unwrap();
        symlink(w.join("real.py"), w.join("linked.py")).unwrap();
        let (has_py, has_pytest) = python_ruff_and_pytest_flags(w);
        assert!(has_py);
        assert!(!has_pytest);
    }

    #[cfg(unix)]
    #[test]
    fn python_pytest_flag_sees_symlinked_test_module() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::tempdir().unwrap();
        let w = tmp.path();
        std::fs::write(w.join("impl.py"), "def test_x():\n    assert True\n").unwrap();
        symlink(w.join("impl.py"), w.join("test_linked.py")).unwrap();
        let (has_py, has_pytest) = python_ruff_and_pytest_flags(w);
        assert!(has_py);
        assert!(has_pytest);
    }
}
