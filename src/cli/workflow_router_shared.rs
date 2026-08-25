use std::path::Path;

use crate::artifacts::{RunArtifacts, SessionDotfileBackups};
#[cfg(test)]
use crate::cli::format_workspace_gate_failure;
use crate::nested_budget_scopes::BudgetScopeLayer;

use crate::repo_checks::{RepoGateOutput, run_repo_workspace_gates};

#[must_use]
pub(crate) fn effective_max_loops(max_loops: usize) -> usize {
    BudgetScopeLayer::effective_outer_loop_iterations(max_loops)
}

#[cfg(test)]
pub(crate) fn prefer_gate_outcome_over_summarize<T>(
    gate: Result<T, String>,
    summarize: Result<(), String>,
) -> Result<T, String> {
    match gate {
        Err(e) => Err(e),
        Ok(v) => summarize.map(|()| v),
    }
}

pub(crate) fn clear_quality_gates_log_for_next_agent(
    artifacts: &RunArtifacts,
) -> Result<(), String> {
    crate::gate_loop_session::set_quality_gates_just_ran(false);
    crate::artifacts::ensure_quality_gates_log_file(artifacts).map_err(|e| e.to_string())
}

fn restore_session_dotfiles_for_gates(
    work_dir: &Path,
    session_dotfile_backups: &SessionDotfileBackups,
    restore_malvin_checks: bool,
) -> Result<(), String> {
    if restore_malvin_checks {
        session_dotfile_backups.restore(work_dir)
    } else {
        session_dotfile_backups.restore_excluding_malvin_checks(work_dir)
    }
}

pub(crate) fn run_router_workspace_gates(
    artifacts: &RunArtifacts,
    session_dotfile_backups: &SessionDotfileBackups,
    restore_malvin_checks: bool,
) -> Result<(), String> {
    let work_dir = artifacts.work_dir.as_path();
    restore_session_dotfiles_for_gates(work_dir, session_dotfile_backups, restore_malvin_checks)?;
    crate::session_dotfile_backup::repair_invalid_malvin_home_config_on_disk(work_dir)?;
    clear_quality_gates_log_for_next_agent(artifacts)?;
    let gate_result = run_repo_workspace_gates(
        work_dir,
        RepoGateOutput::Tagged,
        Some(artifacts.run_dir.as_path()),
    );
    crate::gate_loop_session::set_quality_gates_just_ran(match &gate_result {
        Ok(()) => true,
        Err(detail) => {
            crate::repo_checks::is_gate_failure_error(detail) && detail.contains("failed (exit")
        }
    });
    let restore_result = restore_session_dotfiles_for_gates(
        work_dir,
        session_dotfile_backups,
        restore_malvin_checks,
    );
    prefer_gate_outcome_over_post_gate_cleanup(gate_result, restore_result)
}

pub(crate) fn prefer_gate_outcome_over_post_gate_cleanup(
    gate_result: Result<(), String>,
    restore_result: Result<(), String>,
) -> Result<(), String> {
    gate_result?;
    restore_result
}

#[cfg(test)]
pub(crate) fn post_router_session_gates(
    command: &str,
    artifacts: &RunArtifacts,
    session_dotfile_backups: &SessionDotfileBackups,
    restore_malvin_checks: bool,
) -> Result<(), String> {
    if run_router_workspace_gates(artifacts, session_dotfile_backups, restore_malvin_checks).is_ok()
    {
        return Ok(());
    }
    let review_path = artifacts.artifact_review_md();
    if let Some(parent) = review_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(review_path, b"Checks do not pass\n").map_err(|e| e.to_string())?;
    Err(format_workspace_gate_failure(
        command,
        "workspace quality gates did not pass after the router session",
    ))
}
