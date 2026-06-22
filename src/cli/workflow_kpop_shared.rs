use std::collections::HashMap;
use std::path::Path;

use crate::artifacts::{RunArtifacts, SessionDotfileBackups};
use crate::cli::format_workspace_gate_failure;
use crate::output::{MALVIN_WHO, print_stdout_line};

#[path = "workflow_kpop_render.rs"]
mod workflow_kpop_render;

pub(crate) use workflow_kpop_render::{
    render_kpop_program_request, render_kpop_program_request_creative,
};
use crate::repo_checks::{RepoGateOutput, run_repo_workspace_gates};

#[must_use]
pub(crate) fn effective_max_loops(max_loops: usize) -> usize {
    max_loops.max(1)
}

/// Prefer a gate-loop (or discovery) outcome over a summarize-session error.
pub(crate) fn prefer_gate_outcome_over_summarize<T>(
    gate: Result<T, String>,
    summarize: Result<(), String>,
) -> Result<T, String> {
    match gate {
        Err(e) => Err(e),
        Ok(v) => summarize.map(|()| v),
    }
}

#[must_use]
pub(crate) fn gate_kpop_loop_iterations(max_loops: usize) -> usize {
    let base = effective_max_loops(max_loops);
    if crate::acp::test_no_real_agent_enabled() {
        return base;
    }
    base.saturating_add(1)
}

pub(crate) fn kpop_program_context(
    work_dir: &Path,
    scope_constraints: &str,
    artifacts: &RunArtifacts,
) -> Result<HashMap<String, String>, String> {
    let quality_gates =
        crate::repo_gates::prompt_quality_gates_markdown_ephemeral(work_dir)?;
    let mut context = HashMap::new();
    context.insert(
        "scope_constraints".to_string(),
        scope_constraints.trim().to_string(),
    );
    context.insert("quality_gates".to_string(), quality_gates);
    context.insert(
        "quality_gates_path".to_string(),
        crate::format_prompt_path(
            &artifacts.quality_gates_log_path(),
            &artifacts.work_dir,
        ),
    );
    Ok(context)
}

pub(crate) fn kpop_workflow_context(
    artifacts: &RunArtifacts,
    workflow: &str,
) -> Result<HashMap<String, String>, String> {
    kpop_workflow_context_with_gates(artifacts, workflow, true)
}

pub(crate) fn kpop_workflow_context_without_gates(
    artifacts: &RunArtifacts,
    workflow: &str,
) -> Result<HashMap<String, String>, String> {
    kpop_workflow_context_with_gates(artifacts, workflow, false)
}

fn kpop_workflow_context_with_gates(
    artifacts: &RunArtifacts,
    workflow: &str,
    include_quality_gates: bool,
) -> Result<HashMap<String, String>, String> {
    let mut context = crate::orchestrator::workflow_context_paths_only(artifacts, workflow);
    if include_quality_gates {
        context.insert(
            "quality_gates".to_string(),
            crate::repo_gates::prompt_quality_gates_markdown_ephemeral(&artifacts.work_dir)?,
        );
    }
    Ok(context)
}

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

pub fn write_checks_do_not_pass_for_artifacts(artifacts: &RunArtifacts) -> Result<(), String> {
    write_checks_do_not_pass_to_review_path(&artifacts.artifact_review_md())
}

pub(crate) fn clear_quality_gates_log_for_next_agent(artifacts: &RunArtifacts) -> Result<(), String> {
    crate::artifacts::ensure_quality_gates_log_file(artifacts).map_err(|e| e.to_string())
}

pub(crate) fn gate_iteration_context(
    base: &HashMap<String, String>,
    artifacts: &RunArtifacts,
    exp_log_path: &Path,
    iteration: usize,
) -> HashMap<String, String> {
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

#[allow(dead_code)] // unit tests and kiss coverage stringify references
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

pub(crate) fn run_kpop_workspace_gates(
    artifacts: &RunArtifacts,
    session_dotfile_backups: &SessionDotfileBackups,
    restore_malvin_checks: bool,
) -> Result<(), String> {
    let work_dir = artifacts.work_dir.as_path();
    restore_session_dotfiles_for_gates(work_dir, session_dotfile_backups, restore_malvin_checks)?;
    // Carry-forward backups may still hold kiss-clamp damage; repair on disk before executing gates.
    crate::session_dotfile_backup::repair_clamp_damaged_dotfiles_on_disk(work_dir)?;
    crate::repo_gates::canonical_ignore::reconcile_workspace_ignore_files(work_dir)?;
    clear_quality_gates_log_for_next_agent(artifacts)?;
    let gate_result = run_repo_workspace_gates(
        work_dir,
        RepoGateOutput::Tagged,
        Some(artifacts.run_dir.as_path()),
    );
    // Gate prep (e.g. `kiss clamp`) may mutate dotfiles during the run; rewind disk so
    // outer retries and the next iteration snapshot cannot anchor off re-damaged files.
    let restore_result =
        restore_session_dotfiles_for_gates(work_dir, session_dotfile_backups, restore_malvin_checks);
    // Final restore re-applies pre-session ignore drift; re-apply canonical ops/ exclusion.
    let reconcile_result =
        crate::repo_gates::canonical_ignore::reconcile_workspace_ignore_files(work_dir);
    prefer_gate_outcome_over_post_gate_cleanup(gate_result, restore_result, reconcile_result)
}

pub(crate) fn prefer_gate_outcome_over_post_gate_cleanup(
    gate_result: Result<(), String>,
    restore_result: Result<(), String>,
    reconcile_result: Result<(), String>,
) -> Result<(), String> {
    gate_result?;
    restore_result?;
    reconcile_result
}

pub(crate) fn post_kpop_session_gates(
    command: &str,
    artifacts: &RunArtifacts,
    session_dotfile_backups: &SessionDotfileBackups,
    restore_malvin_checks: bool,
) -> Result<(), String> {
    if run_kpop_workspace_gates(artifacts, session_dotfile_backups, restore_malvin_checks).is_ok() {
        return Ok(());
    }
    write_checks_do_not_pass_for_artifacts(artifacts)?;
    Err(format_workspace_gate_failure(
        command,
        "workspace quality gates did not pass after the kpop session",
    ))
}

pub(crate) fn print_kpop_session_log_line(artifacts: &RunArtifacts, exp_log_path: &Path) {
    let kpop_id = crate::malvin_short_id();
    let log_line = crate::cli::bug_id_lookup_kpop::kpop_log_line(
        &kpop_id,
        &artifacts.work_dir,
        &artifacts.run_dir,
        exp_log_path,
    );
    print_stdout_line(MALVIN_WHO, &log_line);
}

pub(crate) async fn finish_kpop_acp_session(
    artifacts: &RunArtifacts,
    session_dotfile_backups: &SessionDotfileBackups,
    restore_malvin_checks: bool,
) -> Result<(), String> {
    let restore_res = if restore_malvin_checks {
        session_dotfile_backups.restore(&artifacts.work_dir)
    } else {
        session_dotfile_backups.restore_excluding_malvin_checks(&artifacts.work_dir)
    };
    crate::acp_post_run::merge_acp_with_custom_restore_and_check_abort(
        Ok(()),
        restore_res,
        &artifacts.artifact_result_md(),
    )
}
#[cfg(test)]
#[path = "workflow_kpop_shared_test.rs"]
mod workflow_kpop_shared_test;
