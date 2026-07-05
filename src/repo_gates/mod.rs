#![allow(clippy::missing_errors_doc)]

pub(crate) mod gate_command_match;

use std::path::Path;

pub use crate::workspace_paths::MALVIN_CHECKS_REL as MALVIN_CHECKS_FILE;

#[must_use]
pub fn should_run_workspace_gates(work_dir: &Path) -> bool {
    work_dir.join(".git").is_dir()
        || crate::resolve_malvin_checks_path(work_dir).is_file()
        || crate::is_malvin_workspace(work_dir)
}

pub(crate) const fn builtin_gate_command_lines(_work_dir: &Path) -> Vec<String> {
    Vec::new()
}

pub fn gate_command_lines(work_dir: &Path) -> Result<Vec<String>, String> {
    let checks_path = crate::resolve_malvin_checks_path(work_dir);
    if !checks_path.is_file() {
        return Err(format!(
            "{} is missing (quality gates must be listed in .malvin/checks)",
            checks_path.display()
        ));
    }
    load_malvin_checks(&checks_path)
}

pub use gate_command_match::command_matches_malvin_checks_gate;

fn copy_legacy_checks_if_present(
    work_dir: &Path,
    checks_path: &Path,
) -> Result<bool, String> {
    let legacy = crate::legacy_malvin_checks_path(work_dir);
    if !legacy.is_file() {
        return Ok(false);
    }
    if let Some(parent) = checks_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    std::fs::copy(&legacy, checks_path).map_err(|e| {
        format!(
            "copy legacy {} -> {}: {e}",
            legacy.display(),
            checks_path.display()
        )
    })?;
    Ok(true)
}

fn write_builtin_checks_file(checks_path: &Path, lines: &[String]) -> Result<(), String> {
    let mut content = lines.join("\n");
    content.push('\n');
    if let Some(parent) = checks_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    std::fs::write(checks_path, content)
        .map_err(|e| format!("write {}: {e}", checks_path.display()))
}

pub fn ensure_default_malvin_checks_file(work_dir: &Path) -> Result<(), String> {
    let checks_path = crate::malvin_checks_path(work_dir);
    if checks_path.is_file() {
        return Ok(());
    }
    if copy_legacy_checks_if_present(work_dir, &checks_path)? {
        return Ok(());
    }
    let lines = builtin_gate_command_lines(work_dir);
    if lines.is_empty() {
        return Ok(());
    }
    write_builtin_checks_file(&checks_path, &lines)
}

pub fn ensure_default_malvin_config_file(work_dir: &Path) -> Result<(), String> {
    crate::malvin_config_file::ensure_malvin_config_file(work_dir)
}

mod prompt_markdown;

pub use prompt_markdown::{
    prompt_quality_gates_markdown, prompt_quality_gates_markdown_ephemeral,
};
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

pub(crate) fn parse_malvin_checks_text(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(std::string::ToString::to_string)
        .collect()
}

pub fn load_malvin_checks(checks_path: &Path) -> Result<Vec<String>, String> {
    let raw = std::fs::read_to_string(checks_path)
        .map_err(|e| format!("read {}: {e}", checks_path.display()))?;
    Ok(parse_malvin_checks_text(&raw))
}

#[cfg(test)]
#[path = "checks_test_helpers.rs"]
pub(crate) mod checks_test_helpers;

#[cfg(test)]
#[path = "checks_test_helpers_test.rs"]
mod checks_test_helpers_test;

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "git_worktree_tests.rs"]
mod git_worktree_tests;

#[cfg(test)]
#[path = "repo_gates_kiss_cov_tests.rs"]
mod repo_gates_kiss_cov_tests;

#[cfg(test)]
#[path = "tests_git_root_layout.rs"]
mod tests_git_root_layout;

#[cfg(test)]
#[path = "tests_command_match.rs"]
mod tests_command_match;
