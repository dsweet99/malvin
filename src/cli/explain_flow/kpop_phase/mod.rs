//! Explain Review/Plan: one in-process `KPop` session each (not `run_kpop_engine`).

mod chat_rules;

pub(crate) use chat_rules::{
    explain_kpop_chat_rules, EXPLAIN_PHASE_PLAN, EXPLAIN_PHASE_REVIEW,
};

#[cfg(test)]
pub(crate) use chat_rules::{PLAN_CHAT_RULES, REVIEW_CHAT_RULES};

use crate::acp::{AgentError, CoderPromptOptions};
use crate::agent_backend::AgentBackend;
use crate::artifacts::{ensure_explain_phase_exp_log_file, SessionDotfileBackups};
use crate::cli::workflow_kpop_shared::{gate_iteration_context, print_kpop_session_log_line};
use crate::cli::{SharedOpts, WorkflowCliOptions};
use crate::kpop_engine::KPopEnginePrepared;
use crate::prompt_stratification::{join_labeled_strata, PromptStratum};
use crate::prompts::{render_header, PromptError};
use crate::run_timing::TimingPhase;

pub(crate) struct ExplainKpopPhaseParams<'a> {
    #[allow(dead_code)] // kept for call-site symmetry with pre-reuse API
    pub shared: &'a SharedOpts,
    #[allow(dead_code)]
    pub workflow: WorkflowCliOptions,
    pub prepared: &'a KPopEnginePrepared,
    pub request_text: &'a str,
    pub max_hypotheses: usize,
    pub outer_iteration: usize,
    pub phase: &'a str,
    #[allow(dead_code)]
    pub run_timing: &'a std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>,
    /// Open coder session reused across Review/Plan/Work (caller owns begin/end).
    pub client: &'a mut AgentBackend,
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
    mut params: ExplainKpopPhaseParams<'_>,
) -> Result<ExplainKpopPhaseResult, String> {
    run_explain_kpop_phase_once(&mut params).await
}

pub(crate) async fn run_explain_kpop_phase_once(
    params: &mut ExplainKpopPhaseParams<'_>,
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
    let (chat, backups) = run_phase_coder_prompt(params, &prompt, &exp_log_path).await?;
    Ok(ExplainKpopPhaseResult {
        chat,
        backups,
        exp_log_path,
    })
}

async fn run_phase_coder_prompt(
    params: &mut ExplainKpopPhaseParams<'_>,
    prompt: &str,
    exp_log_path: &std::path::Path,
) -> Result<(String, SessionDotfileBackups), String> {
    let work_dir = params.prepared.artifacts().work_dir.as_path();
    let log_label = format!("explain_{}", params.phase);
    let session_dotfile_backups =
        SessionDotfileBackups::snapshot_after_ensuring_home_config(work_dir)?;
    let client = &mut *params.client;
    let prompt_result = {
        let mut prompt_result = client
            .run_coder_prompt(
                prompt,
                params.prepared.artifacts().log_path(&log_label).as_path(),
                &log_label,
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
    };
    let chat = client
        .last_coder_prompt_agent_response()
        .unwrap_or_default();
    finalize_phase_prompt(work_dir, &session_dotfile_backups, prompt_result, chat).await
}

async fn finalize_phase_prompt(
    work_dir: &std::path::Path,
    session_dotfile_backups: &SessionDotfileBackups,
    prompt_result: Result<(), AgentError>,
    chat: String,
) -> Result<(String, SessionDotfileBackups), String> {
    let post_agent_backups = if prompt_result.is_ok() {
        Some(SessionDotfileBackups::snapshot_after_ensuring_home_config(
            work_dir,
        )?)
    } else {
        None
    };
    session_dotfile_backups.restore_excluding_malvin_checks(work_dir)?;
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
