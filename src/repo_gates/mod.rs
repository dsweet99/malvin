#![allow(clippy::missing_errors_doc)]

pub(crate) mod discover_py;
pub mod discover_init_checks;
pub(crate) mod discover_init_checks_signals;
#[cfg(test)]
#[path = "discover_init_checks_fixtures.rs"]
mod discover_init_checks_fixtures;
pub mod init_discovery;
pub(crate) mod init_discovery_validate;
pub(crate) mod sandbox_safe;

#[cfg(test)]
#[path = "discover_init_checks_tests.rs"]
mod discover_init_checks_tests;

#[cfg(test)]
#[path = "discover_init_checks_merge_tests.rs"]
mod discover_init_checks_merge_tests;

use std::path::Path;
use std::process::Stdio;
use std::sync::OnceLock;

use discover_py::python_ruff_and_pytest_flags;

pub use crate::workspace_paths::MALVIN_CHECKS_REL as MALVIN_CHECKS_FILE;

pub const KISSIGNORE_FILE: &str = ".kissignore";

pub const KISSCONFIG_FILE: &str = ".kissconfig";

pub const KISS_CHECK_COMMAND: &str = "kiss check";

pub const DEFAULT_PYTEST_CHECK: &str = "pytest -sv tests";

pub const DEFAULT_RUST_CLIPPY: &str =
    "cargo clippy -j 1 --all-targets --all-features -- -D warnings -W clippy::cargo";

pub const DEFAULT_RUST_TEST: &str = "cargo test";

pub const DEFAULT_RUST_NEXTEST: &str = "cargo nextest run";

pub const DEFAULT_RUST_NEXTEST_PARTITION_1: &str = "cargo nextest run --partition hash:1/2";

pub const DEFAULT_RUST_NEXTEST_PARTITION_2: &str = "cargo nextest run --partition hash:2/2";

static CARGO_NEXTEST_AVAILABLE: OnceLock<bool> = OnceLock::new();

#[must_use]
pub fn cargo_nextest_available(work_dir: &Path) -> bool {
    let _ = work_dir;
    *CARGO_NEXTEST_AVAILABLE.get_or_init(|| {
        crate::malvin_sandbox::malvin_std_command("cargo")
            .args(["nextest", "--version"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    })
}

#[must_use]
pub fn default_rust_test_command(work_dir: &Path) -> &'static str {
    if cargo_nextest_available(work_dir) {
        DEFAULT_RUST_NEXTEST
    } else {
        DEFAULT_RUST_TEST
    }
}

#[must_use]
pub fn rust_test_gate_command_lines(work_dir: &Path) -> Vec<String> {
    if cargo_nextest_available(work_dir) {
        vec![
            DEFAULT_RUST_NEXTEST_PARTITION_1.to_string(),
            DEFAULT_RUST_NEXTEST_PARTITION_2.to_string(),
        ]
    } else {
        vec![DEFAULT_RUST_TEST.to_string()]
    }
}

pub use sandbox_safe::sandbox_safe_gate_commands;

#[must_use]
pub fn should_run_workspace_gates(work_dir: &Path) -> bool {
    work_dir.join(".git").is_dir()
        || crate::malvin_checks_path(work_dir).is_file()
        || crate::is_malvin_workspace(work_dir)
}

pub(crate) fn builtin_gate_command_lines(work_dir: &Path) -> Vec<String> {
    let mut out = vec![KISS_CHECK_COMMAND.to_string()];
    let (has_py, has_pytest) = python_ruff_and_pytest_flags(work_dir);
    if has_py {
        out.push("ruff check .".to_string());
    }
    if has_pytest {
        out.push(DEFAULT_PYTEST_CHECK.to_string());
    }
    if work_dir.join("Cargo.toml").is_file() {
        out.push(DEFAULT_RUST_CLIPPY.to_string());
        out.extend(rust_test_gate_command_lines(work_dir));
    }
    out
}

pub fn gate_command_lines(work_dir: &Path) -> Result<Vec<String>, String> {
    let checks_path = crate::malvin_checks_path(work_dir);
    if checks_path.is_file() {
        return load_malvin_checks(&checks_path);
    }
    Ok(builtin_gate_command_lines(work_dir))
}

/// Overwrite `.malvin/checks` with language/tooling builtins (for init `--force` rediscovery).
pub fn refresh_provisional_malvin_checks_file(work_dir: &Path) -> Result<(), String> {
    let checks_path = crate::malvin_checks_path(work_dir);
    if checks_path.is_file() {
        std::fs::remove_file(&checks_path)
            .map_err(|e| format!("remove {}: {e}", checks_path.display()))?;
    }
    ensure_default_malvin_checks_file(work_dir)
}

pub fn ensure_default_malvin_checks_file(work_dir: &Path) -> Result<(), String> {
    let checks_path = crate::malvin_checks_path(work_dir);
    if checks_path.is_file() {
        return Ok(());
    }
    let lines = builtin_gate_command_lines(work_dir);
    let mut content = lines.join("\n");
    if !content.is_empty() {
        content.push('\n');
    }
    if let Some(parent) = checks_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    std::fs::write(&checks_path, content)
        .map_err(|e| format!("write {}: {e}", checks_path.display()))
}

pub fn ensure_default_malvin_config_file(work_dir: &Path) -> Result<(), String> {
    crate::malvin_config_file::ensure_malvin_config_file(work_dir)
}

pub fn gate_command_lines_for_workspace_run(work_dir: &Path) -> Result<Vec<String>, String> {
    ensure_default_malvin_checks_file(work_dir)?;
    let lines = load_malvin_checks(&crate::malvin_checks_path(work_dir))?;
    Ok(sandbox_safe_gate_commands(&lines))
}

/// Markdown list of quality gate commands for prompt substitution (`{{ quality_gates }}`).
///
/// Reads **only** [`.malvin/checks`]. Returns an error if that file is not a regular file.
/// Callers that need defaults materialized first should run [`ensure_default_malvin_checks_file`].
pub fn prompt_quality_gates_markdown(work_dir: &Path) -> Result<String, String> {
    let checks_path = crate::malvin_checks_path(work_dir);
    if !checks_path.is_file() {
        return Err(format!(
            "{} is missing (expected a regular file before expanding {{ quality_gates }})",
            checks_path.display()
        ));
    }
    let lines = load_malvin_checks(&checks_path)?;
    Ok(format_quality_gates_markdown(&sandbox_safe_gate_commands(&lines)))
}

/// Materializes default `.malvin/checks` only while building prompt markdown, then restores prior state.
///
/// Avoids leaving an untracked `.malvin/checks` when the workspace had none before the call.
pub fn prompt_quality_gates_markdown_ephemeral(work_dir: &Path) -> Result<String, String> {
    use crate::session_dotfile_backup::{
        backup_workspace_malvin_checks_if_present, restore_workspace_malvin_checks_backup,
    };
    let backup = backup_workspace_malvin_checks_if_present(work_dir)?;
    let result = (|| {
        ensure_default_malvin_checks_file(work_dir)?;
        prompt_quality_gates_markdown(work_dir)
    })();
    restore_workspace_malvin_checks_backup(work_dir, &backup)?;
    result
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
mod tests;
