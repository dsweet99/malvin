//! Post-LGTM finish + startup emit for explain.

use crate::cli::error_run_log;
use crate::cli::run_emit::{emit_run_startup_sequence, RunStartupEmitOpts};
use crate::cli::{SharedOpts, WorkflowCliOptions};
use crate::kpop_engine::finish_kpop_engine_after_pass;

use super::outputs::validate_explain_output;
use super::run_startup::ExplainKpopPrepared;

pub(super) struct ExplainSuccessInput<'a> {
    pub prepared: &'a ExplainKpopPrepared,
    pub shared: &'a SharedOpts,
    pub workflow: WorkflowCliOptions,
    pub tex_path: &'a std::path::Path,
    pub pdf_path: &'a std::path::Path,
    pub agent_ran: bool,
    pub run_timing: &'a std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>,
}

pub(super) fn emit_explain_startup(
    shared: &SharedOpts,
    prepared: &ExplainKpopPrepared,
) -> Result<(), String> {
    emit_run_startup_sequence(
        &prepared.inner.artifacts,
        RunStartupEmitOpts::from_shared(shared, true),
        &prepared.inner.startup_emit_request,
    )
}

pub(super) async fn finish_explain_success(input: ExplainSuccessInput<'_>) -> Result<(), String> {
    let ExplainSuccessInput {
        prepared,
        shared,
        workflow,
        tex_path,
        pdf_path,
        agent_ran,
        run_timing,
    } = input;
    validate_explain_output(tex_path, pdf_path)?;
    let summarize_res = crate::cli::kpop_summarize::run_outer_loop_summarize_if_warranted(
        &crate::cli::kpop_summarize::OuterLoopSummarizeParams {
            agent_ran,
            shared,
            workflow,
            store: prepared.inner.store(),
            artifacts: prepared.inner.artifacts(),
            model: &shared.model,
        },
    )
    .await;
    let gate_r = finish_kpop_engine_after_pass(shared, &prepared.inner, agent_ran, Some(run_timing));
    let r = crate::cli::workflow_kpop_shared::prefer_gate_outcome_over_summarize(gate_r, summarize_res);
    if r.is_ok() {
        error_run_log::clear_command_error_run_dir();
    }
    let _ = &prepared.inner.malvin_checks_backup;
    r
}

#[cfg(test)]
mod finish_cov;
