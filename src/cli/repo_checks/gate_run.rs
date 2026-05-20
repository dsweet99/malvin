use std::path::Path;
use std::process::Command;

use super::command_support::{apply_fake_path_if_present, run_command_failure, run_command_for};
use super::gate_log::{emit_repo_gate_line, try_append_command_output};
use super::kissconfig_warn::warn_kissconfig_test_coverage_if_needed;
use super::types::{RepoGateFailure, RepoGateOutput};

/// Workspace quality gates for CLI workflows (`code`, `do`, `kpop`, `bug`, `tidy`, …).
///
/// Runs workspace preparation (`kiss clamp` when applicable) before gate lines.
/// When `.malvin_checks` is absent, writes the same default gate lines that
/// [`repo_gates::gate_command_lines`] would return for a missing file, then runs each non-empty
/// line from `.malvin_checks` in order. Does not run `pre-commit`.
/// With `run_log_dir: Some(path)`, gate output is also appended to `path/quality_gates.log`.
pub fn run_repo_workspace_gates(
    work_dir: &Path,
    output: RepoGateOutput,
    run_log_dir: Option<&Path>,
) -> Result<(), String> {
    use crate::artifacts::{
        backup_workspace_malvin_checks_if_present, restore_workspace_malvin_checks_backup,
    };
    let malvin_checks_backup = backup_workspace_malvin_checks_if_present(work_dir)?;
    let result = run_repo_workspace_gates_with_details(work_dir, output, run_log_dir)
        .map_err(RepoGateFailure::into_error);
    restore_workspace_malvin_checks_backup(work_dir, &malvin_checks_backup)?;
    result
}

/// Same as [`run_repo_workspace_gates`] except workspace preparation skips the `kiss clamp` step.
///
/// Used by `malvin do --repo-gates`, which must not create or rewrite `.kissconfig` implicitly.
pub fn run_repo_workspace_gates_no_kiss_clamp(
    work_dir: &Path,
    output: RepoGateOutput,
    run_log_dir: Option<&Path>,
) -> Result<(), String> {
    use crate::artifacts::{
        backup_workspace_malvin_checks_if_present, restore_workspace_malvin_checks_backup,
    };
    let malvin_checks_backup = backup_workspace_malvin_checks_if_present(work_dir)?;
    let result = run_repo_workspace_gates_no_kiss_clamp_with_details(work_dir, output, run_log_dir)
        .map_err(RepoGateFailure::into_error);
    restore_workspace_malvin_checks_backup(work_dir, &malvin_checks_backup)?;
    result
}

pub fn run_repo_workspace_gates_with_details(
    work_dir: &Path,
    output: RepoGateOutput,
    run_log_dir: Option<&Path>,
) -> Result<(), RepoGateFailure> {
    prepare_repo_workspace_with_details(work_dir, output, run_log_dir, true)?;
    run_quality_gates_with_details(work_dir, output, run_log_dir)
}

pub fn run_repo_workspace_gates_no_kiss_clamp_with_details(
    work_dir: &Path,
    output: RepoGateOutput,
    run_log_dir: Option<&Path>,
) -> Result<(), RepoGateFailure> {
    prepare_repo_workspace_with_details(work_dir, output, run_log_dir, false)?;
    run_quality_gates_with_details(work_dir, output, run_log_dir)
}

#[cfg(test)]
pub fn prepare_repo_workspace(
    work_dir: &Path,
    output: RepoGateOutput,
    run_log_dir: Option<&Path>,
) -> Result<(), String> {
    prepare_repo_workspace_with_details(work_dir, output, run_log_dir, true)
        .map_err(RepoGateFailure::into_error)
}

fn prepare_repo_workspace_with_details(
    work_dir: &Path,
    output: RepoGateOutput,
    run_log_dir: Option<&Path>,
    kiss_clamp_prep: bool,
) -> Result<(), RepoGateFailure> {
    if kiss_clamp_prep {
        ensure_kiss_clamp_if_needed_with_details(work_dir, output, run_log_dir)?;
    }
    warn_kissconfig_test_coverage_if_needed(work_dir, output, run_log_dir);
    Ok(())
}

fn ensure_kiss_clamp_if_needed_with_details(
    work_dir: &Path,
    output: RepoGateOutput,
    run_log_dir: Option<&Path>,
) -> Result<(), RepoGateFailure> {
    let kissconfig = work_dir.join(".kissconfig");
    if kissconfig.exists() || !source_like_files_present(work_dir) {
        return Ok(());
    }
    emit_repo_gate_line(
        output,
        "Running `kiss clamp` (existing code without .kissconfig)",
        run_log_dir,
    );
    let mut command = Command::new(run_command_for("kiss"));
    command.arg("clamp").current_dir(work_dir);
    apply_fake_path_if_present(&mut command);
    let output = command
        .output()
        .map_err(|e| RepoGateFailure::Message(format!("`kiss clamp` failed to start: {e}")))?;
    try_append_command_output(run_log_dir, "kiss clamp", &output);
    if output.status.success() {
        Ok(())
    } else {
        Err(run_command_failure("kiss clamp", &output))
    }
}

pub fn scan_for_extension_handles_symlink_cycles(root: &Path) -> bool {
    crate::source_detect::has_extension_files(root, "rs")
        || crate::source_detect::has_extension_files(root, "py")
}

pub fn source_like_files_present(root: &Path) -> bool {
    scan_for_extension_handles_symlink_cycles(root)
        || crate::source_detect::has_workspace_marker_files(root)
}

fn run_quality_gates_with_details(
    work_dir: &Path,
    output: RepoGateOutput,
    run_log_dir: Option<&Path>,
) -> Result<(), RepoGateFailure> {
    let commands = crate::repo_gates::gate_command_lines_for_workspace_run(work_dir)
        .map_err(RepoGateFailure::Message)?;
    run_malvin_checks_with_details(work_dir, output, run_log_dir, &commands)
}

fn run_malvin_checks_with_details(
    work_dir: &Path,
    output: RepoGateOutput,
    run_log_dir: Option<&Path>,
    commands: &[String],
) -> Result<(), RepoGateFailure> {
    for command in commands.iter().filter(|c| !c.trim().is_empty()) {
        run_shell_command_line_with_details(work_dir, output, run_log_dir, command)?;
    }
    Ok(())
}

const fn shell_binary() -> (&'static str, &'static str) {
    if cfg!(windows) {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    }
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
    let mut command = Command::new(shell);
    command.arg(arg).arg(command_line).current_dir(work_dir);
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
