//! Deterministic `.malvin/checks` discovery from repo signals for `malvin init`.

use std::path::Path;

use super::init_discovery_validate::validate_checks_command_lines;
use super::{DEFAULT_PYTEST_CHECK, KISS_CHECK_COMMAND};
use crate::malvin_checks_path;

pub use super::discover_init_checks_signals::{
    dedupe_check_lines, discover_init_check_commands, makefile_gate_targets,
    precommit_hook_entries,
};

use super::discover_init_checks_signals::canonical_tool;

fn lines_contain_ruff(lines: &[String]) -> bool {
    lines_contain_tool(lines, "ruff")
}

fn lines_contain_tool(lines: &[String], tool: &str) -> bool {
    lines.iter().any(|l| canonical_tool(l) == tool)
}

fn precommit_ruff_and_pytest(root: &Path) -> (Option<String>, Option<String>) {
    let mut ruff = None;
    let mut pytest = None;
    for entry in precommit_hook_entries(root) {
        let trimmed = entry.trim();
        if ruff.is_none() && canonical_tool(trimmed) == "ruff" {
            ruff = Some(trimmed.to_string());
        }
        if pytest.is_none() && canonical_tool(trimmed) == "pytest" {
            pytest = Some(trimmed.to_string());
        }
    }
    (ruff, pytest)
}

fn write_checks_lines(checks_path: &Path, lines: &[String]) -> Result<(), String> {
    std::fs::write(checks_path, format!("{}\n", lines.join("\n")))
        .map_err(|e| format!("write {}: {e}", checks_path.display()))
}

/// Add Python gates from init's `.pre-commit-config.yaml` when builtins skipped them
/// (no `.py` / test modules yet). Keeps Rust `Cargo.toml` builtins intact.
pub fn augment_init_checks_with_precommit_python_gates(root: &Path) -> Result<(), String> {
    let checks_path = malvin_checks_path(root);
    if !checks_path.is_file() {
        return Ok(());
    }
    let mut lines = super::load_malvin_checks(&checks_path)?;
    let (ruff_from_precommit, pytest_from_precommit) = precommit_ruff_and_pytest(root);
    if !lines_contain_ruff(&lines) {
        if let Some(ruff) = ruff_from_precommit {
            lines.push(ruff);
        }
    }
    if !lines_contain_tool(&lines, "pytest") {
        lines.push(
            pytest_from_precommit.unwrap_or_else(|| DEFAULT_PYTEST_CHECK.to_string()),
        );
    }
    write_checks_lines(&checks_path, &lines)
}

/// Rewrite `.malvin/checks` from repo signals after init discovery `KPop`.
pub fn finalize_init_checks_from_repo(root: &Path) -> Result<(), String> {
    let lines = discover_init_check_commands(root);
    validate_checks_command_lines(root, &lines)?;
    let checks_path = malvin_checks_path(root);
    if let Some(parent) = checks_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    let mut content = lines.join("\n");
    if !content.is_empty() {
        content.push('\n');
    }
    std::fs::write(&checks_path, content)
        .map_err(|e| format!("write {}: {e}", checks_path.display()))
}

/// Whether persisted checks cover deduplicated pre-commit hook entries (ignoring kiss).
#[must_use]
pub fn checks_cover_precommit_signals(root: &Path, lines: &[String]) -> bool {
    let expected = dedupe_check_lines(&precommit_hook_entries(root));
    if expected.is_empty() {
        return true;
    }
    let have: std::collections::HashSet<String> = lines
        .iter()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && l.trim() != KISS_CHECK_COMMAND)
        .collect();
    expected
        .iter()
        .all(|cmd| have.contains(cmd.trim()))
}

#[cfg(test)]
mod augment_helpers_tests {
    use super::*;

    #[test]
    fn lines_contain_ruff_detects_ruff_prefix() {
        assert!(lines_contain_ruff(&[
            "kiss check".to_string(),
            "ruff check .".to_string(),
        ]));
        assert!(!lines_contain_ruff(&["kiss check".to_string()]));
    }

    #[test]
    fn lines_contain_tool_matches_canonical_tool() {
        assert!(lines_contain_tool(
            &["pytest -sv tests".to_string()],
            "pytest"
        ));
        assert!(!lines_contain_tool(&["ruff check .".to_string()], "pytest"));
    }

    #[test]
    fn precommit_ruff_and_pytest_parses_hooks() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join(".pre-commit-config.yaml"),
            "repos:\n- repo: local\n  hooks:\n  - id: ruff\n    entry: ruff check .\n  - id: pytest\n    entry: pytest -sv tests\n",
        )
        .unwrap();
        let (ruff, pytest) = precommit_ruff_and_pytest(tmp.path());
        assert_eq!(ruff.as_deref(), Some("ruff check ."));
        assert_eq!(pytest.as_deref(), Some("pytest -sv tests"));
    }

    #[test]
    fn precommit_ruff_and_pytest_accepts_mixed_case_ruff_entry() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join(".pre-commit-config.yaml"),
            "repos:\n- repo: local\n  hooks:\n  - id: ruff\n    entry: RuFf cHEcK .\n",
        )
        .unwrap();
        let (ruff, pytest) = precommit_ruff_and_pytest(tmp.path());
        assert_eq!(ruff.as_deref(), Some("RuFf cHEcK ."));
        assert!(pytest.is_none());
    }

    #[test]
    fn write_checks_lines_persists_joined_commands() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("checks");
        write_checks_lines(
            &path,
            &["kiss check".to_string(), "ruff check .".to_string()],
        )
        .unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        assert_eq!(text, "kiss check\nruff check .\n");
    }

    #[test]
    fn checks_cover_precommit_signals_matches_hook_entries() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join(".pre-commit-config.yaml"),
            "repos:\n- repo: local\n  hooks:\n  - id: ruff\n    entry: ruff check .\n",
        )
        .unwrap();
        let lines = vec!["ruff check .".to_string(), "kiss check".to_string()];
        assert!(checks_cover_precommit_signals(tmp.path(), &lines));
        assert!(!checks_cover_precommit_signals(tmp.path(), &["kiss check".to_string()]));
    }
}
#[cfg(test)]
#[path = "discover_init_checks_kiss_cov_test.rs"]
mod discover_init_checks_kiss_cov_test;
