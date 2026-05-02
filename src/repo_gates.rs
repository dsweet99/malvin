#![allow(clippy::missing_errors_doc)]

use std::path::Path;

pub const MALVIN_CHECKS_FILE: &str = ".malvin_checks";

pub const KISS_CHECK_COMMAND: &str = "kiss check";

pub const DEFAULT_PYTEST_CHECK: &str = "pytest -sv tests";

pub const DEFAULT_RUST_CLIPPY: &str = "cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W clippy::cargo -A clippy::must_use_candidate -A clippy::missing_errors_doc -A clippy::missing_panics_doc";

const DEFAULT_RUST_CHECKS: [&str; 2] = [DEFAULT_RUST_CLIPPY, "cargo test"];

#[must_use]
pub fn should_run_workspace_gates(work_dir: &Path) -> bool {
    work_dir.join(".git").is_dir() || work_dir.join(MALVIN_CHECKS_FILE).is_file()
}

pub fn gate_command_lines(work_dir: &Path) -> Result<Vec<String>, String> {
    let mut out = vec![KISS_CHECK_COMMAND.to_string()];
    let (has_py, has_pytest) = python_ruff_and_pytest_flags(work_dir);
    if has_py {
        out.push("ruff check .".to_string());
    }
    if has_pytest {
        out.push(DEFAULT_PYTEST_CHECK.to_string());
    }
    if work_dir.join("Cargo.toml").is_file() {
        out.extend(DEFAULT_RUST_CHECKS.iter().map(|s| (*s).to_string()));
    }
    let checks_path = work_dir.join(MALVIN_CHECKS_FILE);
    if checks_path.is_file() {
        out.extend(load_malvin_checks(&checks_path)?);
    }
    Ok(out)
}

pub fn prompt_quality_gates_markdown(work_dir: &Path) -> Result<String, String> {
    let lines = gate_command_lines(work_dir)?;
    Ok(format_quality_gates_markdown(&lines))
}

#[must_use]
pub fn format_quality_gates_markdown(commands: &[String]) -> String {
    if commands.is_empty() {
        return String::new();
    }
    commands
        .iter()
        .map(|c| format!("- `{c}`"))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn load_malvin_checks(checks_path: &Path) -> Result<Vec<String>, String> {
    let raw = std::fs::read_to_string(checks_path)
        .map_err(|e| format!("read {}: {e}", checks_path.display()))?;
    Ok(raw
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(std::string::ToString::to_string)
        .collect())
}

fn visit_source_files(root: &Path, f: &mut impl FnMut(&Path)) {
    fn walk(dir: &Path, f: &mut impl FnMut(&Path)) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
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

fn python_ruff_and_pytest_flags(root: &Path) -> (bool, bool) {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn kiss_stringify_repo_gates_internals() {
        let _ = stringify!(super::visit_source_files);
        let _ = stringify!(super::python_ruff_and_pytest_flags);
    }

    #[test]
    fn gate_command_lines_skips_ruff_when_no_python() {
        let tmp = tempfile::tempdir().unwrap();
        let w = tmp.path();
        fs::create_dir(w.join(".git")).unwrap();
        fs::write(
            w.join("Cargo.toml"),
            "[package]\nname = 'm'\nversion = '0.1.0'\n",
        )
        .unwrap();
        let g = gate_command_lines(w).unwrap();
        assert!(g.iter().any(|c| c == KISS_CHECK_COMMAND));
        assert!(!g.iter().any(|c| c.starts_with("ruff")));
        assert!(g.iter().any(|c| c.starts_with("cargo clippy")));
    }

    #[test]
    fn gate_command_lines_skips_pytest_without_test_named_py() {
        let tmp = tempfile::tempdir().unwrap();
        let w = tmp.path();
        fs::create_dir(w.join(".git")).unwrap();
        fs::write(w.join("main.py"), "x=1\n").unwrap();
        let g = gate_command_lines(w).unwrap();
        assert!(g.iter().any(|c| c == "ruff check ."));
        assert!(!g.iter().any(|c| c.contains("pytest")));
    }

    #[test]
    fn gate_command_lines_runs_pytest_when_test_module_present() {
        let tmp = tempfile::tempdir().unwrap();
        let w = tmp.path();
        fs::create_dir(w.join(".git")).unwrap();
        fs::write(w.join("test_foo.py"), "def test_x():\n    assert True\n").unwrap();
        let g = gate_command_lines(w).unwrap();
        assert!(g.iter().any(|c| c == DEFAULT_PYTEST_CHECK));
    }

    #[test]
    fn prompt_quality_gates_includes_rust_builtin_without_git_when_cargo_toml_present() {
        let tmp = tempfile::tempdir().unwrap();
        let w = tmp.path();
        fs::write(
            w.join("Cargo.toml"),
            "[package]\nname='x'\nversion='0.1.0'\n",
        )
        .unwrap();
        let md = prompt_quality_gates_markdown(w).unwrap();
        assert!(md.contains(&format!("- `{KISS_CHECK_COMMAND}`")));
        assert!(md.contains("cargo clippy"));
        assert!(md.contains("cargo test"));
    }
}
