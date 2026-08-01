//! Inline summarize hooks for gate-kpop outer loops.

use crate::agent_backend::AgentBackend;
use crate::artifacts::RunArtifacts;
use crate::prompts::PromptStore;

/// Context for inline summarize chained after a gate-kpop turn in the same coder session.
pub(crate) struct GateInlineSummarizeCtx<'a> {
    pub client: &'a mut AgentBackend,
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub model: &'a str,
    pub git: bool,
    pub iteration: usize,
    pub total_iterations: usize,
}

/// Runs inline summarize inside an open gate-kpop coder session when warranted.
pub(crate) async fn maybe_run_gate_inline_summarize(
    ctx: GateInlineSummarizeCtx<'_>,
) -> Result<(), String> {
    if !super::should_inline_outer_loop_summarize_on_gate_iteration(
        ctx.iteration,
        ctx.total_iterations,
    ) {
        return Ok(());
    }
    super::run_inline_summarize_coder_prompt(
        ctx.client,
        ctx.store,
        ctx.artifacts,
        crate::workflow_context::PromptModelOpts::new(ctx.model, ctx.git),
    )
    .await
}
