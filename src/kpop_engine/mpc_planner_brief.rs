//! Brief reset, done detection, and prompt assembly for the MPC planner.

use std::path::{Path, PathBuf};

use crate::artifacts::RunArtifacts;
use crate::mpc_planning_brief::MpcPlanningBriefAspect;
use crate::prompt_stratification::{join_labeled_strata, PromptStratum, WorkflowRenderContext};
use crate::prompts::{PromptError, PromptStore};
use crate::kpop_progression::mpc_declared_done;

pub(crate) fn mpc_planner_iteration_log_path(artifacts: &RunArtifacts, iteration: usize) -> PathBuf {
    artifacts.log_path(&format!("mpc_planner_{iteration}"))
}

pub(crate) fn user_brief_baseline_path(artifacts: &RunArtifacts) -> PathBuf {
    artifacts.run_dir.join("user_request_baseline.md")
}

pub(crate) fn reset_user_brief_before_planner(
    artifacts: &RunArtifacts,
    context: &WorkflowRenderContext,
) -> Result<(), String> {
    let _aspect = MpcPlanningBriefAspect::BriefBaselineReset;
    let brief_path = crate::workflow_context::resolve_user_brief_path(artifacts, context);
    let baseline_path = user_brief_baseline_path(artifacts);
    if !baseline_path.is_file() {
        let brief_text = std::fs::read_to_string(&brief_path).map_err(|e| {
            format!(
                "failed to read user brief {}: {e}",
                brief_path.display()
            )
        })?;
        std::fs::write(&baseline_path, &brief_text).map_err(|e| e.to_string())?;
    }
    let baseline = std::fs::read_to_string(&baseline_path).map_err(|e| {
        format!(
            "failed to read user brief baseline {}: {e}",
            baseline_path.display()
        )
    })?;
    std::fs::write(&brief_path, &baseline).map_err(|e| e.to_string())
}

pub(crate) fn user_brief_declares_mpc_done(path: &Path) -> Result<bool, String> {
    let _aspect = MpcPlanningBriefAspect::DoneMarkerDetection;
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read user brief {}: {e}", path.display()))?;
    Ok(mpc_declared_done(&text))
}

#[must_use]
pub(crate) fn mpc_planner_exp_log_path(artifacts: &RunArtifacts) -> PathBuf {
    let _aspect = MpcPlanningBriefAspect::HypothesisLogPath;
    artifacts.run_dir.join("_kpop").join("mpc_planner_log.md")
}

pub(crate) fn build_mpc_planner_context(
    base: &WorkflowRenderContext,
    artifacts: &RunArtifacts,
) -> WorkflowRenderContext {
    if crate::acp::test_no_real_agent_enabled() {
        let mut ctx = base.clone();
        ctx.insert(
            "exp_log".to_string(),
            "./_kpop/mpc_planner_log.md".to_string(),
        );
        return ctx;
    }
    let mut ctx = base.clone();
    let exp_log_path = mpc_planner_exp_log_path(artifacts);
    let exp_log = crate::format_prompt_path(&exp_log_path, &artifacts.work_dir);
    ctx.insert("exp_log".to_string(), exp_log);
    ctx.insert(
        "current_state".to_string(),
        crate::current_state::format_current_state(
            artifacts.work_dir.as_path(),
            None,
            Some(artifacts),
        ),
    );
    ctx
}

/// Assemble `header.md` + `kpop_common.md` + `mpc_planner.md`.
///
/// # Errors
///
/// Returns `Err` when a prompt template cannot be rendered.
pub(crate) fn build_mpc_planner_prompt(
    store: &PromptStore,
    context: &WorkflowRenderContext,
) -> Result<String, String> {
    let _aspect = MpcPlanningBriefAspect::BriefAppendProtocol;
    if crate::acp::test_no_real_agent_enabled() {
        let map = context.as_map();
        let user_req = map
            .get("user_request_path")
            .cloned()
            .unwrap_or_else(|| "./user_request.md".to_string());
        return Ok(format!(
            "# MPC Request\n\nUser request (read this file):\n\n`{user_req}`\n"
        ));
    }
    let map = context.as_map();
    let header = store
        .render_prompt_only("header.md", map)
        .map_err(|e: PromptError| e.0)?;
    let common = store
        .render_prompt_only("kpop_common.md", map)
        .map_err(|e: PromptError| e.0)?;
    let body = store
        .render_prompt_only("mpc_planner.md", map)
        .map_err(|e: PromptError| e.0)?;
    Ok(join_labeled_strata([
        (PromptStratum::WorkflowHeader, header),
        (PromptStratum::EmbeddedTemplate, common),
        (PromptStratum::GateLoopBlock, body),
    ]))
}
