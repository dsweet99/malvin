//! Inline summarize hooks for gate-kpop and malvin-kpop outer loops.

use crate::agent_backend::AgentBackend;
use crate::artifacts::RunArtifacts;
use crate::gate_kpop_workflow::GateLoopBehavior;
use crate::prompts::PromptStore;

/// Context for inline summarize at the end of a `malvin kpop` outer-loop iteration.
pub(crate) struct InlineSummarizeOnKpopLoopCtx<'a> {
    pub client: &'a mut AgentBackend,
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub agent_loop: usize,
    pub max_loops: usize,
    pub will_exit_after_this_loop: bool,
}

/// Runs inline summarize on the last qualifying `malvin kpop` loop iteration.
pub(crate) async fn maybe_run_inline_summarize_on_kpop_loop(
    ctx: InlineSummarizeOnKpopLoopCtx<'_>,
) -> Result<(), String> {
    if !super::should_inline_outer_loop_summarize_on_kpop_loop(
        ctx.agent_loop,
        ctx.max_loops,
        ctx.will_exit_after_this_loop,
    ) {
        return Ok(());
    }
    let work_dir = ctx.artifacts.work_dir.as_path();
    ctx.client
        .begin_coder_session(work_dir)
        .await
        .map_err(|e| e.to_string())?;
    let summarize_res = super::run_inline_summarize_coder_prompt(
        ctx.client,
        ctx.store,
        ctx.artifacts,
        "malvin kpop",
    )
    .await;
    let end_res = ctx.client.end_coder_session().await.map_err(|e| e.to_string());
    crate::acp_post_run::prefer_primary_over_secondary(summarize_res, end_res, "end coder session")
}

/// Context for inline summarize chained after a gate-kpop turn in the same coder session.
pub(crate) struct GateInlineSummarizeCtx<'a> {
    pub client: &'a mut AgentBackend,
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub malvin_command: &'a str,
    pub iteration: usize,
    pub total_iterations: usize,
    pub consecutive_solved_entering: usize,
    pub behavior: GateLoopBehavior,
}

/// Runs inline summarize inside an open gate-kpop coder session when warranted.
pub(crate) async fn maybe_run_gate_inline_summarize(
    ctx: GateInlineSummarizeCtx<'_>,
) -> Result<(), String> {
    if !super::should_inline_outer_loop_summarize_on_gate_iteration(
        ctx.iteration,
        ctx.total_iterations,
        ctx.consecutive_solved_entering,
        ctx.behavior,
    ) {
        return Ok(());
    }
    super::run_inline_summarize_coder_prompt(
        ctx.client,
        ctx.store,
        ctx.artifacts,
        ctx.malvin_command,
    )
    .await
}
#[cfg(test)]
#[path = "kpop_summarize_inline_test.rs"]
mod kpop_summarize_inline_test;#[cfg(test)]
#[path = "kpop_summarize_inline_kiss_cov_test.rs"]
mod kpop_summarize_inline_kiss_cov_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<GateInlineSummarizeCtx> = None;
        let _: Option<InlineSummarizeOnKpopLoopCtx> = None;
    }
}
