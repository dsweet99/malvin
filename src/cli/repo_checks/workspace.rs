use malvin::repo_gates;
use std::path::Path;
use std::process::Command;

use super::command_support::{
    apply_fake_path_if_present, run_command_failure, run_command_for,
};
use super::emit::emit_repo_gate_line;
use super::kissconfig_warn::warn_kissconfig_test_coverage_if_needed;
use super::types::{RepoGateFailure, RepoGateOutput};

/// Workspace quality gates for CLI workflows (`code`, `do`, `kpop`, `tidy`, …).
///
/// Calls [`prepare_repo_workspace`] first (`kiss clamp` when applicable).
/// Runs [`repo_gates::gate_command_lines`]: when `.malvin_checks` exists, only non-empty lines from
/// that file (in order); otherwise built-ins derived from the workspace tree. Does not run `pre-commit`.
/// Never creates or edits `.malvin_checks`.
/// With `run_log_dir: Some(path)`, each gate line is also appended to `path/quality_checks.log`.
pub fn run_repo_workspace_gates(
    work_dir: &Path,
    output: RepoGateOutput,
    run_log_dir: Option<&Path>,
) -> Result<(), String> {
    run_repo_workspace_gates_with_details(work_dir, output, run_log_dir)
        .map_err(RepoGateFailure::into_error)
}

pub fn run_repo_workspace_gates_with_details(
    work_dir: &Path,
    output: RepoGateOutput,
    run_log_dir: Option<&Path>,
) -> Result<(), RepoGateFailure> {
    prepare_repo_workspace_with_details(work_dir, output, run_log_dir)?;
    run_quality_gates_with_details(work_dir, output, run_log_dir)
}

pub fn prepare_repo_workspace(
    work_dir: &Path,
    output: RepoGateOutput,
    run_log_dir: Option<&Path>,
) -> Result<(), String> {
    prepare_repo_workspace_with_details(work_dir, output, run_log_dir)
        .map_err(RepoGateFailure::into_error)
}

fn prepare_repo_workspace_with_details(
    work_dir: &Path,
    output: RepoGateOutput,
    run_log_dir: Option<&Path>,
) -> Result<(), RepoGateFailure> {
    ensure_kiss_clamp_if_needed_with_details(work_dir, output, run_log_dir)?;
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
    if output.status.success() {
        Ok(())
    } else {
        Err(run_command_failure("kiss clamp", &output))
    }
}

pub fn source_like_files_present(root: &Path) -> bool {
    crate::cli::kiss_clamp::has_source_files(root)
}

fn run_quality_gates_with_details(
    work_dir: &Path,
    output: RepoGateOutput,
    run_log_dir: Option<&Path>,
) -> Result<(), RepoGateFailure> {
    let commands =
        repo_gates::gate_command_lines(work_dir).map_err(RepoGateFailure::Message)?;
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
    emit_repo_gate_line(
        output,
        &format!("Running `{command_line}`"),
        run_log_dir,
    );
    let (shell, arg) = shell_binary();
    let mut command = Command::new(shell);
    command.arg(arg).arg(command_line).current_dir(work_dir);
    apply_fake_path_if_present(&mut command);
    let output = command
        .output()
        .map_err(|e| RepoGateFailure::Message(format!("`{command_line}` failed to start: {e}")))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(run_command_failure(command_line, &output))
    }
}

#[cfg(test)]
mod kiss_stringify_workspace {
    #[test]
    fn kiss_stringify_repo_checks_workspace_internals() {
        let _ = stringify!(super::run_repo_workspace_gates_with_details);
        let _ = stringify!(super::prepare_repo_workspace_with_details);
        let _ = stringify!(super::ensure_kiss_clamp_if_needed_with_details);
        let _ = stringify!(super::run_quality_gates_with_details);
        let _ = stringify!(super::run_malvin_checks_with_details);
        let _ = stringify!(super::shell_binary);
        let _ = stringify!(super::run_shell_command_line_with_details);
    }
}
