//! Auto-run `kiss clamp` when existing code lacks `.kissconfig`.

use std::path::Path;
use std::process::Command;

use crate::cli::repo_checks::{RepoGateOutput, emit_repo_gate_stdout_line};

/// Returns true if the directory contains source files (`.rs`, `.py`) or project markers.
fn has_source_files(dir: &Path) -> bool {
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
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ext == "rs" || ext == "py" {
                        return true;
                    }
                }
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name == "Cargo.toml"
                        || name == "pyproject.toml"
                        || name == "requirements.txt"
                    {
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

/// If existing code is detected but `.kissconfig` is absent, run `kiss clamp`.
pub fn ensure_kiss_clamp_if_needed(work_dir: &Path, output: RepoGateOutput) -> Result<(), String> {
    let kissconfig = work_dir.join(".kissconfig");
    if kissconfig.exists() {
        return Ok(());
    }
    if !has_source_files(work_dir) {
        return Ok(());
    }
    emit_repo_gate_stdout_line(
        output,
        "Running `kiss clamp` (existing code without .kissconfig)",
    );
    let status = Command::new("kiss")
        .arg("clamp")
        .current_dir(work_dir)
        .status()
        .map_err(|e| format!("`kiss clamp` failed to start: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("`kiss clamp` failed".to_string())
    }
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

    #[test]
    fn ensure_kiss_clamp_skips_when_kissconfig_exists() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join(".kissconfig"), "").unwrap();
        std::fs::write(tmp.path().join("main.rs"), "fn main() {}").unwrap();
        assert!(ensure_kiss_clamp_if_needed(tmp.path(), RepoGateOutput::Tagged).is_ok());
    }

    #[test]
    fn ensure_kiss_clamp_skips_when_no_source_files() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(ensure_kiss_clamp_if_needed(tmp.path(), RepoGateOutput::Plain).is_ok());
    }
}
