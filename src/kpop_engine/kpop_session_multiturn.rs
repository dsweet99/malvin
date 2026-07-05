use crate::acp::AgentError;
use crate::kpop_turn_prompts::KpopTurnPrompts;
use crate::cli::workflow_kpop_shared::gate_iteration_context;
use crate::prompt_stratification::WorkflowRenderContext;
use crate::prompts::render_priors_mbc2_prompt;

use super::kpop_session::{
    KPopEngineMultiturnCtx, finalize_kpop_engine_turn, restore_kpop_engine_session_dotfiles,
    run_kpop_engine_coder_turn,
};

const fn make_turn_prompts<'a>(
    ctx: &'a KPopEngineMultiturnCtx<'_>,
    iter_ctx: &'a WorkflowRenderContext,
) -> KpopTurnPrompts<'a> {
    let prepared = ctx.iteration.loop_params.prepared;
    KpopTurnPrompts {
        store: prepared.store(),
        base: iter_ctx,
        prepend_rules_once: false,
    }
}

pub(super) fn iter_context(ctx: &KPopEngineMultiturnCtx<'_>) -> WorkflowRenderContext {
    let prepared = ctx.iteration.loop_params.prepared;
    gate_iteration_context(
        prepared.context(),
        prepared.artifacts(),
        &ctx.iteration.exp_log_path,
        ctx.iteration.iteration,
    )
}

pub(super) fn build_prompt_priors(ctx: &KPopEngineMultiturnCtx<'_>) -> Result<String, String> {
    let prepared = ctx.iteration.loop_params.prepared;
    let ic = iter_context(ctx);
    render_priors_mbc2_prompt(prepared.store(), ic.as_map()).map_err(|e| e.0)
}

fn build_prompt_a(ctx: &KPopEngineMultiturnCtx<'_>) -> Result<String, String> {
    let ic = iter_context(ctx);
    make_turn_prompts(ctx, &ic).kpop_engine_prompt_a()
}

fn build_prompt_b(ctx: &KPopEngineMultiturnCtx<'_>) -> Result<String, String> {
    let ic = iter_context(ctx);
    make_turn_prompts(ctx, &ic).kpop_engine_prompt_b()
}

fn build_prompt_c(ctx: &KPopEngineMultiturnCtx<'_>) -> Result<String, String> {
    let ic = iter_context(ctx);
    make_turn_prompts(ctx, &ic).kpop_engine_prompt_c()
}

async fn send_phase(
    ctx: &mut KPopEngineMultiturnCtx<'_>,
    prompt: &str,
    work_dir: &std::path::Path,
    log_path: &std::path::Path,
) -> Result<(), AgentError> {
    run_kpop_engine_coder_turn(ctx, prompt, work_dir, log_path).await
}

fn mpc_plan_is_done(ctx: &KPopEngineMultiturnCtx<'_>) -> bool {
    let prepared = ctx.iteration.loop_params.prepared;
    crate::kpop_progression::mpc_plan_declares_done(
        &crate::artifacts::mpc_plan_path(prepared.artifacts()),
    )
    .unwrap_or(false)
}

pub(super) async fn run_kpop_engine_multiturn(
    ctx: &mut KPopEngineMultiturnCtx<'_>,
) -> Result<Option<crate::artifacts::SessionDotfileBackups>, String> {
    let prepared = ctx.iteration.loop_params.prepared;
    let work_dir = prepared.artifacts().work_dir.as_path();
    let log_path = prepared.artifacts().log_path("kpop");

    let result = run_phases(ctx, work_dir, log_path.as_path()).await;

    if let Err(e) = result {
        let _ = restore_kpop_engine_session_dotfiles(ctx);
        ctx.iteration.client.end_coder_session().await.ok();
        return Err(e.0);
    }

    let post_agent_backups = Some(
        crate::artifacts::SessionDotfileBackups::snapshot_after_ensuring_home_config(work_dir)?,
    );
    finalize_kpop_engine_turn(ctx, work_dir, Ok(())).await?;
    Ok(post_agent_backups)
}

async fn run_phases(
    ctx: &mut KPopEngineMultiturnCtx<'_>,
    work_dir: &std::path::Path,
    log_path: &std::path::Path,
) -> Result<(), AgentError> {
    let prompt_priors = build_prompt_priors(ctx).map_err(AgentError)?;
    send_phase(ctx, &prompt_priors, work_dir, log_path).await?;

    let prompt_a = build_prompt_a(ctx).map_err(AgentError)?;
    send_phase(ctx, &prompt_a, work_dir, log_path).await?;

    let prompt_b = build_prompt_b(ctx).map_err(AgentError)?;
    send_phase(ctx, &prompt_b, work_dir, log_path).await?;

    if !mpc_plan_is_done(ctx) {
        let prompt_c = build_prompt_c(ctx).map_err(AgentError)?;
        send_phase(ctx, &prompt_c, work_dir, log_path).await?;
    }
    Ok(())
}
