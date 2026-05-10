#![allow(clippy::missing_errors_doc)]

pub(crate) mod discover_py;

use std::path::Path;

use discover_py::python_ruff_and_pytest_flags;

pub const MALVIN_CHECKS_FILE: &str = ".malvin_checks";

pub const KISS_CHECK_COMMAND: &str = "kiss check";

pub const DEFAULT_PYTEST_CHECK: &str = "pytest -sv tests";

pub const DEFAULT_RUST_CLIPPY: &str = "cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W clippy::cargo";

const DEFAULT_RUST_CHECKS: [&str; 2] = [DEFAULT_RUST_CLIPPY, "cargo test"];

#[must_use]
pub fn should_run_workspace_gates(work_dir: &Path) -> bool {
    work_dir.join(".git").is_dir() || work_dir.join(MALVIN_CHECKS_FILE).is_file()
}

fn builtin_gate_command_lines(work_dir: &Path) -> Vec<String> {
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
    out
}

pub fn gate_command_lines(work_dir: &Path) -> Result<Vec<String>, String> {
    let checks_path = work_dir.join(MALVIN_CHECKS_FILE);
    if checks_path.is_file() {
        return load_malvin_checks(&checks_path);
    }
    Ok(builtin_gate_command_lines(work_dir))
}

pub fn ensure_default_malvin_checks_file(work_dir: &Path) -> Result<(), String> {
    let checks_path = work_dir.join(MALVIN_CHECKS_FILE);
    if checks_path.is_file() {
        return Ok(());
    }
    let lines = builtin_gate_command_lines(work_dir);
    let mut content = lines.join("\n");
    if !content.is_empty() {
        content.push('\n');
    }
    std::fs::write(&checks_path, content)
        .map_err(|e| format!("write {}: {e}", checks_path.display()))
}

pub fn gate_command_lines_for_workspace_run(work_dir: &Path) -> Result<Vec<String>, String> {
    ensure_default_malvin_checks_file(work_dir)?;
    load_malvin_checks(&work_dir.join(MALVIN_CHECKS_FILE))
}

/// Markdown list of quality gate commands for prompt substitution (`{{ quality_gates }}`).
///
/// Reads **only** [`MALVIN_CHECKS_FILE`]. Returns an error if that file is not a regular file.
/// Callers that need defaults materialized first should run [`ensure_default_malvin_checks_file`].
pub fn prompt_quality_gates_markdown(work_dir: &Path) -> Result<String, String> {
    let checks_path = work_dir.join(MALVIN_CHECKS_FILE);
    if !checks_path.is_file() {
        return Err(format!(
            "{} is missing (expected a regular file before expanding {{ quality_gates }})",
            checks_path.display()
        ));
    }
    let lines = load_malvin_checks(&checks_path)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn kiss_stringify_repo_gates_internals() {
        let _ = stringify!(super::builtin_gate_command_lines);
        let _ = stringify!(crate::repo_gates::discover_py::visit_source_files);
        let _ = stringify!(crate::repo_gates::discover_py::python_ruff_and_pytest_flags);
        let _ = stringify!(super::ensure_default_malvin_checks_file);
        let _ = stringify!(super::gate_command_lines_for_workspace_run);
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
    fn gate_command_lines_uses_only_malvin_checks_when_present() {
        let tmp = tempfile::tempdir().unwrap();
        let w = tmp.path();
        fs::create_dir(w.join(".git")).unwrap();
        fs::write(
            w.join("Cargo.toml"),
            "[package]\nname = 'm'\nversion = '0.1.0'\n",
        )
        .unwrap();
        fs::write(w.join(".malvin_checks"), "custom-a\ncustom-b\n").unwrap();
        let g = gate_command_lines(w).unwrap();
        assert_eq!(g, vec!["custom-a".to_string(), "custom-b".to_string()]);
        assert!(!g.iter().any(|c| c == KISS_CHECK_COMMAND));
    }

    #[test]
    fn ensure_default_malvin_checks_file_writes_builtin_lines() {
        let tmp = tempfile::tempdir().unwrap();
        let w = tmp.path();
        fs::create_dir(w.join(".git")).unwrap();
        fs::write(
            w.join("Cargo.toml"),
            "[package]\nname = 'm'\nversion = '0.1.0'\n",
        )
        .unwrap();
        let checks_path = w.join(MALVIN_CHECKS_FILE);
        assert!(!checks_path.exists());
        let expected = gate_command_lines(w).unwrap();
        ensure_default_malvin_checks_file(w).unwrap();
        assert!(checks_path.is_file());
        assert_eq!(load_malvin_checks(&checks_path).unwrap(), expected);
        ensure_default_malvin_checks_file(w).unwrap();
        assert_eq!(load_malvin_checks(&checks_path).unwrap(), expected);
    }

    #[test]
    fn gate_command_lines_for_workspace_run_matches_file_after_ensure() {
        let tmp = tempfile::tempdir().unwrap();
        let w = tmp.path();
        fs::create_dir(w.join(".git")).unwrap();
        fs::write(
            w.join("Cargo.toml"),
            "[package]\nname = 'm'\nversion = '0.1.0'\n",
        )
        .unwrap();
        let a = gate_command_lines_for_workspace_run(w).unwrap();
        let b = gate_command_lines(w).unwrap();
        assert_eq!(a, b);
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
        ensure_default_malvin_checks_file(w).unwrap();
        let md = prompt_quality_gates_markdown(w).unwrap();
        assert!(md.contains(&format!("- `{KISS_CHECK_COMMAND}`")));
        assert!(md.contains("cargo clippy"));
        assert!(md.contains("cargo test"));
    }

    #[test]
    fn prompt_quality_gates_markdown_errors_when_malvin_checks_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let w = tmp.path();
        let err = prompt_quality_gates_markdown(w).unwrap_err();
        assert!(
            err.contains("is missing"),
            "unexpected error message: {err}"
        );
    }
}
