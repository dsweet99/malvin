mod review_lgtm;

pub(crate) use review_lgtm::explain_review_chat_is_lgtm;

use crate::cli::error_run_log;
use crate::cli::{SharedOpts, WorkflowCliOptions};
use crate::run_timing::attach_new_run_timing;

use super::finish::{emit_explain_startup, finish_explain_success, ExplainSuccessInput};
use super::kpop_phase::{
    run_explain_kpop_phase, ExplainKpopPhaseParams, EXPLAIN_PHASE_PLAN, EXPLAIN_PHASE_REVIEW,
};
use super::outputs::{products_nonempty, resolve_explain_output_paths};
use super::prep::{
    explain_plan_request, explain_work_request, ExplainKpopRequestInput, ExplainWorkPromptParts,
};
use super::run_startup::{prepare_explain_kpop_run, ExplainKpopPrepared};
use super::work::{run_explain_work, ExplainWorkParams};
use super::{effective_explain_max_loops, ExplainArgs};

pub async fn run_explain(
    explain: &mut ExplainArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let prepared = prepare_explain_run(explain, shared, workflow)?;
    error_run_log::set_command_error_run_dir(Some(prepared.inner.artifacts.run_dir.clone()));
    emit_explain_startup(shared, &prepared)?;
    let mut run_timing_slot = None;
    let run_timing = attach_new_run_timing(&mut run_timing_slot);
    let max_outer = effective_explain_max_loops(explain.max_loops);
    for outer in 1..=max_outer {
        if let Some(done) = run_outer_iteration(OuterIterationCtx {
            explain,
            shared,
            workflow,
            prepared: &prepared,
            outer,
            run_timing: &run_timing,
        })
        .await?
        {
            return done;
        }
    }
    // Closing review: the last work pass may have paid gaps after that loop's review.
    let mut closing = OuterIterationCtx {
        explain,
        shared,
        workflow,
        prepared: &prepared,
        outer: max_outer.saturating_add(1),
        run_timing: &run_timing,
    };
    let closing_chat = run_review_phase(&closing).await?;
    if explain_review_chat_is_lgtm(&closing_chat) {
        return finish_on_lgtm(&mut closing).await;
    }
    Err(format!(
        "malvin explain: exhausted --max-loops={max_outer} without LGTM review"
    ))
}

fn prepare_explain_run(
    explain: &mut ExplainArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<ExplainKpopPrepared, String> {
    let _request = explain
        .request
        .as_ref()
        .ok_or_else(|| "malvin explain: missing required REQUEST (text or path)".to_string())?;
    let prepared = prepare_explain_kpop_run(
        explain.request.as_ref(),
        &explain.out_path,
        explain.out_path_explicit,
        super::run_startup::ExplainKpopPrepareOpts {
            workflow,
            model: &shared.model,
            git: shared.git,
        },
    )?;
    if explain.out_path_explicit {
        explain.out_path =
            crate::cli::default_output_path::path_relative_to_cwd(&prepared.tex_path)?;
    }
    Ok(prepared)
}

struct OuterIterationCtx<'a> {
    explain: &'a mut ExplainArgs,
    shared: &'a SharedOpts,
    workflow: WorkflowCliOptions,
    prepared: &'a ExplainKpopPrepared,
    outer: usize,
    run_timing: &'a std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>,
}

async fn run_outer_iteration(
    mut ctx: OuterIterationCtx<'_>,
) -> Result<Option<Result<(), String>>, String> {
    let review_chat = run_review_phase(&ctx).await?;
    if explain_review_chat_is_lgtm(&review_chat) {
        return Ok(Some(finish_on_lgtm(&mut ctx).await));
    }
    let plan_chat = run_plan_phase(&ctx, &review_chat).await?;
    run_work_phase(&ctx, &review_chat, &plan_chat).await?;
    Ok(None)
}

async fn run_review_phase(ctx: &OuterIterationCtx<'_>) -> Result<String, String> {
    let review = run_explain_kpop_phase(ExplainKpopPhaseParams {
        shared: ctx.shared,
        workflow: ctx.workflow,
        prepared: &ctx.prepared.inner,
        request_text: &ctx.prepared.inner.request_text,
        max_hypotheses: ctx.explain.max_hypotheses,
        outer_iteration: ctx.outer,
        phase: EXPLAIN_PHASE_REVIEW,
        run_timing: ctx.run_timing,
    })
    .await?;
    let chat = review.chat;
    if chat.trim().is_empty() {
        return Err("malvin explain: broken review (empty agent chat)".to_string());
    }
    let _ = review.backups;
    Ok(chat)
}

async fn finish_on_lgtm(ctx: &mut OuterIterationCtx<'_>) -> Result<(), String> {
    let (tex_path, pdf_path) = resolve_explain_output_paths(ctx.prepared)?;
    if !products_nonempty(&tex_path, &pdf_path) {
        return Err(
            "malvin explain: review returned LGTM but tex/pdf products are missing or empty"
                .to_string(),
        );
    }
    if ctx.prepared.auto_out_path {
        ctx.explain.out_path =
            crate::cli::default_output_path::path_relative_to_cwd(&tex_path)?;
    }
    finish_explain_success(ExplainSuccessInput {
        prepared: ctx.prepared,
        shared: ctx.shared,
        workflow: ctx.workflow,
        tex_path: &tex_path,
        pdf_path: &pdf_path,
        agent_ran: true,
        run_timing: ctx.run_timing,
    })
    .await
}

async fn run_plan_phase(ctx: &OuterIterationCtx<'_>, review_chat: &str) -> Result<String, String> {
    let plan_request = explain_plan_request(ctx.prepared.inner.store(), review_chat)?;
    let plan = run_explain_kpop_phase(ExplainKpopPhaseParams {
        shared: ctx.shared,
        workflow: ctx.workflow,
        prepared: &ctx.prepared.inner,
        request_text: &plan_request,
        max_hypotheses: ctx.explain.max_hypotheses,
        outer_iteration: ctx.outer,
        phase: EXPLAIN_PHASE_PLAN,
        run_timing: ctx.run_timing,
    })
    .await?;
    let chat = plan.chat;
    if chat.trim().is_empty() {
        return Err("malvin explain: broken plan (empty agent chat)".to_string());
    }
    let _ = plan.backups;
    Ok(chat)
}

async fn run_work_phase(
    ctx: &OuterIterationCtx<'_>,
    review_chat: &str,
    plan_chat: &str,
) -> Result<(), String> {
    let outputs = super::prep::ExplainResolvedOutputs {
        tex_path: ctx.prepared.tex_path.clone(),
        pdf_path: ctx.prepared.pdf_path.clone(),
    };
    let work_request = explain_work_request(
        ctx.prepared.inner.store(),
        ctx.prepared.inner.artifacts(),
        ExplainWorkPromptParts {
            paths: ExplainKpopRequestInput {
                request_text: "",
                request_work_dir: &ctx.prepared.request_work_dir,
                outputs: &outputs,
                out_path_explicit: !ctx.prepared.auto_out_path,
            },
            review: review_chat,
            plan: plan_chat,
        },
    )?;
    let _backups = run_explain_work(ExplainWorkParams {
        shared: ctx.shared,
        workflow: ctx.workflow,
        prepared: &ctx.prepared.inner,
        work_request: &work_request,
        run_timing: ctx.run_timing,
    })
    .await?;
    Ok(())
}

#[cfg(test)]
#[path = "../../explain_flow_run_loop_tests.rs"]
mod explain_flow_run_loop_tests;

#[cfg(test)]
mod run_loop_cov;
