use std::collections::HashMap;
use std::path::Path;

use crate::artifacts::{RunArtifacts, SessionDotfileBackups};
use crate::cli::format_workspace_gate_failure;
use crate::output::{MALVIN_WHO, print_stdout_line};
use crate::prompts::{PromptError, PromptStore};
use crate::repo_checks::{RepoGateOutput, run_repo_workspace_gates};

#[must_use]
pub(crate) fn effective_max_loops(max_loops: usize) -> usize {
    max_loops.max(1)
}

#[must_use]
pub(crate) fn gate_kpop_loop_iterations(max_loops: usize) -> usize {
    effective_max_loops(max_loops).saturating_add(1)
}

pub(crate) fn kpop_program_context(
    work_dir: &Path,
    scope_constraints: &str,
) -> Result<HashMap<String, String>, String> {
    let quality_gates =
        crate::repo_gates::prompt_quality_gates_markdown_ephemeral(work_dir)?;
    let mut context = HashMap::new();
    context.insert(
        "scope_constraints".to_string(),
        scope_constraints.trim().to_string(),
    );
    context.insert("quality_gates".to_string(), quality_gates);
    Ok(context)
}

pub(crate) fn render_kpop_program_request(
    store: &PromptStore,
    work_dir: &Path,
    constraints_prompt: &str,
    constraints_context: &HashMap<String, String>,
) -> Result<String, String> {
    let scope_constraints = store
        .render_prompt_only(constraints_prompt, constraints_context)
        .map_err(|e: PromptError| e.0)?;
    let context = kpop_program_context(work_dir, &scope_constraints)?;
    store
        .render_prompt_only("kpop_program.md", &context)
        .map(|s| s.trim().to_string())
        .map_err(|e: PromptError| e.0)
}

pub(crate) fn kpop_workflow_context(
    artifacts: &RunArtifacts,
    workflow: &str,
) -> Result<HashMap<String, String>, String> {
    let mut context = crate::orchestrator::workflow_context_paths_only(artifacts, workflow);
    context.insert(
        "quality_gates".to_string(),
        crate::repo_gates::prompt_quality_gates_markdown_ephemeral(&artifacts.work_dir)?,
    );
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
    write_checks_do_not_pass_to_review_path(&artifacts.artifact_review_md())?;
    write_checks_do_not_pass_to_review_path(&artifacts.workspace_review_md())
}

pub(crate) fn run_kpop_workspace_gates(artifacts: &RunArtifacts) -> Result<(), String> {
    run_repo_workspace_gates(
        artifacts.work_dir.as_path(),
        RepoGateOutput::Tagged,
        Some(artifacts.run_dir.as_path()),
    )
}

pub(crate) fn post_kpop_session_gates(
    command: &str,
    artifacts: &RunArtifacts,
) -> Result<(), String> {
    if run_kpop_workspace_gates(artifacts).is_ok() {
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
) -> Result<(), String> {
    crate::acp_post_run::merge_acp_with_workspace_session_restore_and_check_abort(
        Ok(()),
        &artifacts.work_dir,
        session_dotfile_backups,
        &artifacts.artifact_result_md(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_max_loops_is_at_least_one() {
        assert_eq!(effective_max_loops(0), 1);
        assert_eq!(effective_max_loops(3), 3);
    }

    #[test]
    fn gate_kpop_loop_iterations_is_one_plus_max_loops() {
        assert_eq!(gate_kpop_loop_iterations(0), 2);
        assert_eq!(gate_kpop_loop_iterations(5), 6);
    }

    #[test]
    fn kpop_workflow_context_includes_quality_gates() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
        std::fs::write(tmp.path().join(".malvin/checks"), "kiss check\n").expect("checks");
        let artifacts =
            crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
        let ctx = kpop_workflow_context(&artifacts, "code").expect("context");
        assert!(ctx.contains_key("quality_gates"));
    }

    #[test]
    fn write_checks_do_not_pass_for_artifacts_writes_markers() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts =
            crate::artifacts::create_kpop_run_artifacts("tidy", Some(tmp.path())).expect("artifacts");
        write_checks_do_not_pass_for_artifacts(&artifacts).expect("write");
        assert!(artifacts.artifact_review_md().exists());
        assert!(artifacts.workspace_review_md().exists());
    }
}
