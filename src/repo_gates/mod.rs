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

pub fn gate_command_lines(work_dir: &Path) -> Result<Vec<String>, String> {
    let checks_path = crate::resolve_malvin_checks_path(work_dir);
    if !checks_path.is_file() {
        return Err(format!(
            "{} is missing (quality gates must be listed in .malvin/gates)",
            checks_path.display()
        ));
    }
    load_malvin_checks(&checks_path)
}

pub use gate_command_match::command_matches_malvin_checks_gate;

pub fn ensure_default_malvin_config_file(work_dir: &Path) -> Result<(), String> {
    crate::malvin_config_file::ensure_malvin_config_file(work_dir)
}

mod prompt_markdown;

pub use prompt_markdown::{prompt_quality_gates_markdown, prompt_quality_gates_markdown_ephemeral};
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
