//! Explain Review/Plan: one in-process `KPop` session each (not `run_kpop_engine`).

use crate::acp::{AgentError, CoderPromptOptions};
use crate::agent_backend::{agent_backend_set_run_timing, build_agent_backend, AgentBackend};
use crate::artifacts::{ensure_explain_phase_exp_log_file, SessionDotfileBackups};
use crate::cli::workflow_kpop_shared::{gate_iteration_context, print_kpop_session_log_line};
use crate::cli::{SharedOpts, WorkflowCliOptions};
use crate::kpop_engine::KPopEnginePrepared;
use crate::prompt_stratification::{join_labeled_strata, PromptStratum};
use crate::prompts::{render_header, PromptError};
use crate::run_timing::TimingPhase;

pub(crate) const EXPLAIN_PHASE_REVIEW: &str = "review";
pub(crate) const EXPLAIN_PHASE_PLAN: &str = "plan";

pub(crate) const REVIEW_CHAT_RULES: &str = "\
Judge lack-of-satisfaction. Do not edit. The entire agent chat body must be exactly `LGTM` \
(and only LGTM) when nothing fails, or else a failure-focused gap list. Missing/empty products \
⇒ never LGTM. Probe cold entry (first sentence of any early stretch opens on a \
definition, mechanism, or toy before landscape/pressure; a warm earlier stretch \
does not license a cold later opening) and settle-and-stop (later moves not forced \
by what earlier stretches established); fail the review when either appears.
";

pub(crate) const PLAN_CHAT_RULES: &str = "\
Put the plan only in the agent chat body. Do not edit files. Do not echo an executive summary \
or tl;dr to chat.
";

pub(crate) fn explain_kpop_chat_rules(phase: &str) -> &'static str {
    if phase == EXPLAIN_PHASE_PLAN {
        PLAN_CHAT_RULES
    } else {
        REVIEW_CHAT_RULES
    }
}

pub(crate) struct ExplainKpopPhaseParams<'a> {
    pub shared: &'a SharedOpts,
    pub workflow: WorkflowCliOptions,
    pub prepared: &'a KPopEnginePrepared,
    pub request_text: &'a str,
    pub max_hypotheses: usize,
    pub outer_iteration: usize,
    pub phase: &'a str,
    pub run_timing: &'a std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>,
}

pub(crate) struct ExplainKpopPhaseResult {
    pub chat: String,
    pub backups: SessionDotfileBackups,
    #[allow(dead_code)] // retained for callers / diagnostics
    pub exp_log_path: std::path::PathBuf,
}

pub(crate) struct ExplainKpopPromptInput<'a> {
    pub prepared: &'a KPopEnginePrepared,
    pub request_text: &'a str,
    pub exp_log_path: &'a std::path::Path,
    pub outer_iteration: usize,
    pub phase: &'a str,
    pub max_hypotheses: usize,
}

pub(crate) fn build_explain_kpop_phase_prompt(
    input: ExplainKpopPromptInput<'_>,
) -> Result<String, String> {
    let mut ctx = gate_iteration_context(
        input.prepared.context(),
        input.prepared.artifacts(),
        input.exp_log_path,
        input.outer_iteration,
    );
    ctx.insert("want", input.max_hypotheses.to_string());
    ctx.insert("remaining_hypotheses", "0");
    ctx.insert("user_request", input.request_text);
    ctx.insert("explain_kpop_chat_rules", explain_kpop_chat_rules(input.phase));
    render_explain_kpop_strata(input.prepared.store(), ctx.as_map())
}

fn render_explain_kpop_strata(
    store: &crate::prompts::PromptStore,
    map: &std::collections::HashMap<String, String>,
) -> Result<String, String> {
    let header = render_header(store, map).map_err(|e: PromptError| e.0)?;
    let common = store
        .render_prompt_only("explain_kpop_common.md", map)
        .map_err(|e: PromptError| e.0)?;
    let turn = store
        .render_prompt_only("explain_kpop_turn.md", map)
        .map_err(|e: PromptError| e.0)?;
    Ok(join_labeled_strata([
        (PromptStratum::WorkflowHeader, header),
        (PromptStratum::EmbeddedTemplate, common),
        (PromptStratum::GateLoopBlock, turn),
    ]))
}

pub(crate) async fn run_explain_kpop_phase(
    params: ExplainKpopPhaseParams<'_>,
) -> Result<ExplainKpopPhaseResult, String> {
    run_explain_kpop_phase_once(params).await
}

pub(crate) async fn run_explain_kpop_phase_once(
    params: ExplainKpopPhaseParams<'_>,
) -> Result<ExplainKpopPhaseResult, String> {
    let prepared = params.prepared;
    let exp_log_path = ensure_explain_phase_exp_log_file(
        prepared.artifacts(),
        params.outer_iteration,
        params.phase,
    )
    .map_err(|e| e.to_string())?;
    print_kpop_session_log_line(prepared.artifacts(), &exp_log_path);
    let prompt = build_explain_kpop_phase_prompt(ExplainKpopPromptInput {
        prepared,
        request_text: params.request_text,
        exp_log_path: &exp_log_path,
        outer_iteration: params.outer_iteration,
        phase: params.phase,
        max_hypotheses: params.max_hypotheses,
    })?;
    let (chat, backups) = run_phase_coder_session(&params, &prompt, &exp_log_path).await?;
    Ok(ExplainKpopPhaseResult {
        chat,
        backups,
        exp_log_path,
    })
}

fn open_phase_backend(params: &ExplainKpopPhaseParams<'_>) -> Result<AgentBackend, String> {
    let mut client = build_agent_backend(
        params.shared,
        params.workflow,
        params.shared.acp_stdout_markdown_enabled(),
        "explain",
    )
    .map_err(|e| e.to_string())?;
    agent_backend_set_run_timing(&mut client, Some(std::sync::Arc::clone(params.run_timing)));
    client.set_prompts_log_run_dir(Some(params.prepared.artifacts().run_dir.clone()));
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    Ok(client)
}

async fn run_phase_coder_session(
    params: &ExplainKpopPhaseParams<'_>,
    prompt: &str,
    exp_log_path: &std::path::Path,
) -> Result<(String, SessionDotfileBackups), String> {
    let work_dir = params.prepared.artifacts().work_dir.as_path();
    let log_label = format!("explain_{}", params.phase);
    let mut client = open_phase_backend(params)?;
    let session_dotfile_backups =
        SessionDotfileBackups::snapshot_after_ensuring_home_config(work_dir)?;
    client
        .begin_coder_session(work_dir)
        .await
        .map_err(|e| e.to_string())?;
    let prompt_result =
        run_phase_prompt((&mut client, params, prompt, &log_label, exp_log_path)).await;
    let chat = client
        .last_coder_prompt_agent_response()
        .unwrap_or_default();
    finalize_phase_session((
        &mut client,
        work_dir,
        &session_dotfile_backups,
        prompt_result,
        chat,
    ))
    .await
}

type PhasePromptArgs<'a> = (
    &'a mut AgentBackend,
    &'a ExplainKpopPhaseParams<'a>,
    &'a str,
    &'a str,
    &'a std::path::Path,
);

async fn run_phase_prompt(args: PhasePromptArgs<'_>) -> Result<(), AgentError> {
    let (client, params, prompt, log_label, exp_log_path) = args;
    let mut prompt_result = client
        .run_coder_prompt(
            prompt,
            params.prepared.artifacts().log_path(log_label).as_path(),
            log_label,
            CoderPromptOptions {
                llm_phase: Some(TimingPhase::Implement),
                single_attempt: true,
                ..Default::default()
            },
        )
        .await;
    if prompt_result.is_ok() {
        prompt_result = crate::kpop_progression::check_hypothesis_budget(
            exp_log_path,
            params.max_hypotheses,
        )
        .map_err(AgentError);
    }
    prompt_result
}

type PhaseFinalizeArgs<'a> = (
    &'a mut AgentBackend,
    &'a std::path::Path,
    &'a SessionDotfileBackups,
    Result<(), AgentError>,
    String,
);

async fn finalize_phase_session(
    args: PhaseFinalizeArgs<'_>,
) -> Result<(String, SessionDotfileBackups), String> {
    let (client, work_dir, session_dotfile_backups, prompt_result, chat) = args;
    let post_agent_backups = if prompt_result.is_ok() {
        Some(SessionDotfileBackups::snapshot_after_ensuring_home_config(
            work_dir,
        )?)
    } else {
        None
    };
    if let Err(restore_err) = session_dotfile_backups.restore_excluding_malvin_checks(work_dir) {
        client.end_coder_session().await.ok();
        return Err(restore_err);
    }
    client
        .end_coder_session()
        .await
        .map_err(|e| e.to_string())?;
    prompt_result.map_err(|e| e.to_string())?;
    let progress = post_agent_backups.unwrap_or_else(|| session_dotfile_backups.clone());
    Ok((
        chat,
        crate::artifacts::merge_and_sanitize_for_gate_restore(
            session_dotfile_backups,
            &progress,
            work_dir,
        ),
    ))
}

#[cfg(test)]
mod kpop_phase_cov;
