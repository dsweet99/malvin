use std::path::{Path, PathBuf};

use super::command_support::{apply_fake_path_if_present, run_command_failure};
use super::gate_log::{emit_repo_gate_line, try_append_command_output};
use super::types::{RepoGateFailure, RepoGateOutput, repo_gate_failure_to_string};

pub fn run_repo_workspace_gates(
    work_dir: &Path,
    output: RepoGateOutput,
    run_log_dir: Option<&Path>,
) -> Result<(), String> {
    use crate::artifacts::{
        backup_workspace_malvin_checks_if_present, restore_workspace_malvin_checks_backup,
    };
    let malvin_checks_backup = backup_workspace_malvin_checks_if_present(work_dir)?;
    let gate_result = run_repo_workspace_gates_with_details(work_dir, output, run_log_dir)
        .map_err(repo_gate_failure_to_string);
    let restore_result =
        restore_workspace_malvin_checks_backup(work_dir, &malvin_checks_backup);
    prefer_gate_outcome_over_checks_restore(gate_result, restore_result)
}

fn prefer_gate_outcome_over_checks_restore(
    gate_result: Result<(), String>,
    restore_result: Result<(), String>,
) -> Result<(), String> {
    gate_result?;
    restore_result
}

pub fn run_repo_workspace_gates_with_details(
    work_dir: &Path,
    output: RepoGateOutput,
    run_log_dir: Option<&Path>,
) -> Result<(), RepoGateFailure> {
    prepare_repo_workspace_with_details(work_dir)?;
    run_quality_gates_with_details(work_dir, output, run_log_dir)
}

#[cfg(test)]
pub fn prepare_repo_workspace(
    work_dir: &Path,
    output: RepoGateOutput,
    run_log_dir: Option<&Path>,
) -> Result<(), String> {
    let _ = output;
    let _ = run_log_dir;
    prepare_repo_workspace_with_details(work_dir).map_err(repo_gate_failure_to_string)
}

fn prepare_repo_workspace_with_details(work_dir: &Path) -> Result<(), RepoGateFailure> {
    crate::session_dotfile_backup::repair_invalid_malvin_home_config_on_disk(work_dir)
        .map_err(RepoGateFailure::Message)?;
    Ok(())
}

fn run_quality_gates_with_details(
    work_dir: &Path,
    output: RepoGateOutput,
    run_log_dir: Option<&Path>,
) -> Result<(), RepoGateFailure> {
    let commands = crate::repo_gates::gate_command_lines(work_dir)
        .map_err(RepoGateFailure::Message)?;
    run_malvin_checks_with_details(work_dir, output, run_log_dir, &commands)
}

fn run_malvin_checks_with_details(
    work_dir: &Path,
    output: RepoGateOutput,
    run_log_dir: Option<&Path>,
    commands: &[String],
) -> Result<(), RepoGateFailure> {
    crate::agent_phase::enter_verifying();
    let result = (|| {
        for command in commands.iter().filter(|c| !c.trim().is_empty()) {
            run_shell_command_line_with_details(work_dir, output, run_log_dir, command)?;
        }
        Ok(())
    })();
    crate::agent_phase::leave_verifying();
    result
}

const fn shell_binary() -> (&'static str, &'static str) {
    if cfg!(windows) {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    }
}

/// Run `.malvin/checks` lines at the git worktree toplevel when present.
///
/// Checks are resolved from the repo root (see `malvin_checks_path`), and the
/// documented contract is that gate commands also execute there. Using the
/// possibly nested agent `work_dir` as cwd breaks relative commands such as
/// `pytest tests` when the session workspace is a subdirectory.
#[must_use]
pub(crate) fn gate_command_cwd(work_dir: &Path) -> PathBuf {
    crate::git_worktree_toplevel(work_dir).unwrap_or_else(|| work_dir.to_path_buf())
}

fn run_shell_command_line_with_details(
    work_dir: &Path,
    output: RepoGateOutput,
    run_log_dir: Option<&Path>,
    command: &str,
) -> Result<(), RepoGateFailure> {
    let command_line = command.trim();
    if command_line.is_empty() {
        return Ok(());
    }
    emit_repo_gate_line(output, &format!("Running `{command_line}`"), run_log_dir);
    let (shell, arg) = shell_binary();
    let mut command = crate::malvin_sandbox::malvin_std_command(shell);
    command
        .arg(arg)
        .arg(command_line)
        .current_dir(gate_command_cwd(work_dir));
    apply_fake_path_if_present(&mut command);
    let output = command
        .output()
        .map_err(|e| RepoGateFailure::Message(format!("`{command_line}` failed to start: {e}")))?;
    try_append_command_output(run_log_dir, command_line, &output);
    if output.status.success() {
        Ok(())
    } else {
        Err(run_command_failure(command_line, &output))
    }
}

#[cfg(test)]
#[path = "gate_run_tests.rs"]
mod gate_run_tests;
