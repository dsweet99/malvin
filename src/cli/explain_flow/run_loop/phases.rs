//! Outer-loop Review / Plan / Work phases on one open coder session.

use crate::agent_backend::AgentBackend;
use crate::cli::{SharedOpts, WorkflowCliOptions};

use super::super::finish::{finish_explain_success, ExplainSuccessInput};
use super::super::kpop_phase::{
    run_explain_kpop_phase, ExplainKpopPhaseParams, EXPLAIN_PHASE_PLAN, EXPLAIN_PHASE_REVIEW,
};
use super::super::outputs::{products_nonempty, resolve_explain_output_paths};
use super::super::prep::{
    explain_plan_request, explain_work_request, ExplainKpopRequestInput, ExplainWorkPromptParts,
};
use super::super::run_startup::ExplainKpopPrepared;
use super::super::work::{run_explain_work, ExplainWorkParams};
use super::super::ExplainArgs;
use super::explain_review_chat_is_lgtm;

pub(super) struct OuterIterationCtx<'a> {
    pub explain: &'a mut ExplainArgs,
    pub shared: &'a SharedOpts,
    pub workflow: WorkflowCliOptions,
    pub prepared: &'a ExplainKpopPrepared,
    pub outer: usize,
    pub run_timing: &'a std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>,
    pub client: &'a mut AgentBackend,
}

pub(super) struct ExplainOpenSession<'a> {
    pub explain: &'a mut ExplainArgs,
    pub shared: &'a SharedOpts,
    pub workflow: WorkflowCliOptions,
    pub prepared: &'a ExplainKpopPrepared,
    pub run_timing: &'a std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>,
    pub client: &'a mut AgentBackend,
    pub max_outer: usize,
}

pub(super) async fn run_outer_iteration(
    mut ctx: OuterIterationCtx<'_>,
) -> Result<Option<Result<(), String>>, String> {
    let review_chat = run_review_phase(&mut ctx).await?;
    if explain_review_chat_is_lgtm(&review_chat) {
        return Ok(Some(finish_on_lgtm(&mut ctx).await));
    }
    let plan_chat = run_plan_phase(&mut ctx, &review_chat).await?;
    run_work_phase(&mut ctx, &review_chat, &plan_chat).await?;
    Ok(None)
}

pub(super) async fn run_explain_with_open_session(
    session: ExplainOpenSession<'_>,
) -> Result<(), String> {
    let ExplainOpenSession {
        explain,
        shared,
        workflow,
        prepared,
        run_timing,
        client,
        max_outer,
    } = session;
    for outer in 1..=max_outer {
        if let Some(done) = run_outer_iteration(OuterIterationCtx {
            explain,
            shared,
            workflow,
            prepared,
            outer,
            run_timing,
            client,
        })
        .await?
        {
            return done;
        }
    }
    let mut closing = OuterIterationCtx {
        explain,
        shared,
        workflow,
        prepared,
        outer: max_outer.saturating_add(1),
        run_timing,
        client,
    };
    let closing_chat = run_review_phase(&mut closing).await?;
    if explain_review_chat_is_lgtm(&closing_chat) {
        return finish_on_lgtm(&mut closing).await;
    }
    Err(format!(
        "malvin explain: exhausted --max-loops={max_outer} without LGTM review"
    ))
}

async fn run_review_phase(ctx: &mut OuterIterationCtx<'_>) -> Result<String, String> {
    let review = run_explain_kpop_phase(ExplainKpopPhaseParams {
        shared: ctx.shared,
        workflow: ctx.workflow,
        prepared: &ctx.prepared.inner,
        request_text: &ctx.prepared.inner.request_text,
        max_hypotheses: ctx.explain.max_hypotheses,
        outer_iteration: ctx.outer,
        phase: EXPLAIN_PHASE_REVIEW,
        run_timing: ctx.run_timing,
        client: ctx.client,
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

async fn run_plan_phase(
    ctx: &mut OuterIterationCtx<'_>,
    review_chat: &str,
) -> Result<String, String> {
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
        client: ctx.client,
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
    ctx: &mut OuterIterationCtx<'_>,
    review_chat: &str,
    plan_chat: &str,
) -> Result<(), String> {
    let outputs = super::super::prep::ExplainResolvedOutputs {
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
        client: ctx.client,
    })
    .await?;
    Ok(())
}
