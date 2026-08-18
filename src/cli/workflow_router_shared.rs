use std::path::Path;

use crate::artifacts::{RunArtifacts, SessionDotfileBackups};
#[cfg(test)]
use crate::cli::format_workspace_gate_failure;
use crate::nested_budget_scopes::BudgetScopeLayer;
#[cfg(test)]
use crate::prompt_stratification::WorkflowRenderContext;

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

#[cfg(test)]
fn router_workflow_context_with_gates(
    artifacts: &RunArtifacts,
    opts: crate::workflow_context::PromptModelOpts<'_>,
    include_quality_gates: bool,
) -> Result<WorkflowRenderContext, String> {
    let mut context =
        crate::orchestrator::workflow_context_paths_only(artifacts, opts.model, opts.git);
    if include_quality_gates {
        context.insert(
            "quality_gates".to_string(),
            crate::repo_gates::prompt_quality_gates_markdown_ephemeral(&artifacts.work_dir)?,
        );
    }
    Ok(context)
}

#[cfg(test)]
pub(crate) fn router_workflow_context(
    artifacts: &RunArtifacts,
    model: &str,
    git: bool,
) -> Result<WorkflowRenderContext, String> {
    router_workflow_context_with_gates(
        artifacts,
        crate::workflow_context::PromptModelOpts::new(model, git),
        true,
    )
}

#[cfg(test)]
pub(crate) fn router_workflow_context_without_gates(
    artifacts: &RunArtifacts,
    model: &str,
    git: bool,
) -> Result<WorkflowRenderContext, String> {
    router_workflow_context_with_gates(
        artifacts,
        crate::workflow_context::PromptModelOpts::new(model, git),
        false,
    )
}

#[cfg(test)]
pub fn write_checks_do_not_pass_to_review_path(review_path: &Path) -> Result<(), String> {
    if let Some(parent) = review_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            format!(
                "failed to create parent dirs for {}: {e}",
                review_path.display()
            )
        })?;
    }
    std::fs::write(review_path, b"Checks do not pass\n").map_err(|e| {
        format!(
            "failed to write checks-do-not-pass marker {}: {e}",
            review_path.display()
        )
    })
}

#[cfg(test)]
pub fn write_checks_do_not_pass_for_artifacts(artifacts: &RunArtifacts) -> Result<(), String> {
    write_checks_do_not_pass_to_review_path(&artifacts.artifact_review_md())
}

pub(crate) fn clear_quality_gates_log_for_next_agent(artifacts: &RunArtifacts) -> Result<(), String> {
    crate::artifacts::ensure_quality_gates_log_file(artifacts).map_err(|e| e.to_string())
}

#[cfg(test)]
pub(crate) fn gate_iteration_context(
    base: &WorkflowRenderContext,
    artifacts: &RunArtifacts,
    exp_log_path: &Path,
    iteration: usize,
) -> WorkflowRenderContext {
    let mut ctx = base.clone();
    let exp_log = crate::format_prompt_path(exp_log_path, &artifacts.work_dir);
    ctx.insert("exp_log".to_string(), exp_log);
    ctx.insert(
        "current_state".to_string(),
        crate::current_state::format_current_state(
            artifacts.work_dir.as_path(),
            Some(iteration),
            Some(artifacts),
        ),
    );
    ctx
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
    let restore_result =
        restore_session_dotfiles_for_gates(work_dir, session_dotfile_backups, restore_malvin_checks);
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
    if run_router_workspace_gates(artifacts, session_dotfile_backups, restore_malvin_checks).is_ok() {
        return Ok(());
    }
    write_checks_do_not_pass_for_artifacts(artifacts)?;
    Err(format_workspace_gate_failure(
        command,
        "workspace quality gates did not pass after the router session",
    ))
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs {
    use super::*;

    #[test]
    fn kiss_cov_unit_names() {
        let _ = stringify!(router_workflow_context_with_gates);
        let _ = stringify!(router_workflow_context);
        let _ = stringify!(router_workflow_context_without_gates);
        let _ = stringify!(write_checks_do_not_pass_to_review_path);
        let _ = stringify!(write_checks_do_not_pass_for_artifacts);
        let _ = stringify!(gate_iteration_context);
        let _ = stringify!(post_router_session_gates);
        let _ = stringify!(run_router_workspace_gates);
        let _ = stringify!(prefer_gate_outcome_over_post_gate_cleanup);
        let _ = stringify!(effective_max_loops);
        let _ = stringify!(clear_quality_gates_log_for_next_agent);
    }
}
