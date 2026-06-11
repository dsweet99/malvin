//! Post-session summarize agent when `--max-loops` > 1.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::artifacts::{RunArtifacts, SessionDotfileBackups};
use crate::cli::{build_agent, SharedOpts, WorkflowCliOptions};
use crate::cli::workflow_kpop_shared::{effective_max_loops, kpop_workflow_context};
use crate::prompts::{render_header, PromptError, PromptStore};
use crate::run_timing::TimingPhase;

const SUMMARIZE_PROMPT: &str = "kpop_summarize.md";

/// Inputs for [`run_outer_loop_summarize_if_warranted`].
pub(crate) struct OuterLoopSummarizeParams<'a> {
    pub max_loops: usize,
    pub agent_ran: bool,
    pub shared: &'a SharedOpts,
    pub workflow: WorkflowCliOptions,
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub malvin_command: &'a str,
}

/// Inputs for [`code_outer_loop_summarize_params`].
pub(crate) struct CodeOuterLoopSummarizeInputs<'a> {
    pub max_loops: usize,
    pub agent_ran: bool,
    pub shared: &'a SharedOpts,
    pub workflow: WorkflowCliOptions,
}

#[must_use]
pub(crate) const fn code_outer_loop_summarize_params<'a>(
    inputs: CodeOuterLoopSummarizeInputs<'a>,
    prepared: &'a crate::cli::code_flow::CodeKpopPrepared,
) -> OuterLoopSummarizeParams<'a> {
    OuterLoopSummarizeParams {
        max_loops: inputs.max_loops,
        agent_ran: inputs.agent_ran,
        shared: inputs.shared,
        workflow: inputs.workflow,
        store: prepared.store(),
        artifacts: prepared.artifacts(),
        malvin_command: "malvin code",
    }
}

/// Inputs for [`kpop_outer_loop_summarize_params`].
pub(crate) struct KpopOuterLoopSummarizeInputs<'a> {
    pub max_loops: usize,
    pub agent_ran: bool,
    pub shared: &'a SharedOpts,
}

#[must_use]
pub(crate) const fn kpop_outer_loop_summarize_params<'a>(
    inputs: KpopOuterLoopSummarizeInputs<'a>,
    store: &'a PromptStore,
    artifacts: &'a RunArtifacts,
) -> OuterLoopSummarizeParams<'a> {
    OuterLoopSummarizeParams {
        max_loops: inputs.max_loops,
        agent_ran: inputs.agent_ran,
        shared: inputs.shared,
        workflow: WorkflowCliOptions { force: false },
        store,
        artifacts,
        malvin_command: "malvin kpop",
    }
}

/// Whether an outer-loop summarize agent should run after `KPop` sessions complete.
#[must_use]
pub(crate) fn outer_loop_summarize_warranted(max_loops: usize) -> bool {
    effective_max_loops(max_loops) > 1
}

/// Prefer a gate-loop (or discovery) outcome over a summarize-session error.
///
/// Summarize runs before this merge; when the primary workflow failed, that error must
/// not be replaced by a summarize failure.
pub(crate) fn prefer_gate_outcome_over_summarize<T>(
    gate: Result<T, String>,
    summarize: Result<(), String>,
) -> Result<T, String> {
    match gate {
        Err(e) => Err(e),
        Ok(v) => summarize.map(|()| v),
    }
}

pub(crate) fn is_written_exp_log_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|name| {
            name.starts_with("exp_log_")
                && Path::new(name)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        })
}

pub(crate) fn list_written_exp_logs(run_dir: &Path) -> Vec<PathBuf> {
    let kpop_dir = run_dir.join("_kpop");
    let Ok(entries) = std::fs::read_dir(&kpop_dir) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| is_written_exp_log_path(p))
        .collect();
    paths.sort();
    paths
}

pub(crate) fn exp_log_paths_markdown(artifacts: &RunArtifacts) -> String {
    let paths = list_written_exp_logs(&artifacts.run_dir);
    if paths.is_empty() {
        return "  (none yet)\n".to_string();
    }
    paths
        .iter()
        .map(|p| format!("- {}", crate::format_prompt_path(p, &artifacts.work_dir)))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

pub(crate) fn insert_summarize_log_context(
    ctx: &mut HashMap<String, String>,
    artifacts: &RunArtifacts,
    max_loops: usize,
) {
    ctx.insert(
        "kpop_log".to_string(),
        crate::format_prompt_path(&artifacts.log_path("kpop"), &artifacts.work_dir),
    );
    ctx.insert(
        "stdout_log".to_string(),
        crate::format_prompt_path(&artifacts.stdout_log_path(), &artifacts.work_dir),
    );
    ctx.insert(
        "command_log".to_string(),
        crate::format_prompt_path(&artifacts.run_dir.join("command.log"), &artifacts.work_dir),
    );
    ctx.insert("exp_log_paths".to_string(), exp_log_paths_markdown(artifacts));
    ctx.insert(
        "outer_loop_count".to_string(),
        effective_max_loops(max_loops).to_string(),
    );
}

pub(crate) fn render_kpop_summarize_prompt(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    malvin_command: &str,
    max_loops: usize,
) -> Result<String, String> {
    let mut ctx = kpop_workflow_context(artifacts, malvin_command)?;
    insert_summarize_log_context(&mut ctx, artifacts, max_loops);
    let header = render_header(store, &ctx).map_err(|e: PromptError| e.0)?;
    let body = store
        .render_prompt_only(SUMMARIZE_PROMPT, &ctx)
        .map_err(|e: PromptError| e.0)?;
    Ok(format!("{}\n\n{}", header.trim_end(), body.trim_end()))
}

pub(crate) async fn run_summarize_coder_prompt(
    client: &mut crate::acp::AgentClient,
    artifacts: &RunArtifacts,
    prompt: &str,
) -> Result<(), String> {
    client
        .run_coder_prompt(
            prompt,
            &artifacts.log_path("summary"),
            "summary",
            crate::acp::CoderPromptOptions {
                llm_phase: Some(TimingPhase::Implement),
                do_trace_split: None,
                stdout_bracket_label: Some(SUMMARIZE_PROMPT),
                ..Default::default()
            },
        )
        .await
        .map_err(|e| e.to_string())
}

pub(crate) async fn run_summarize_agent_session(
    params: &OuterLoopSummarizeParams<'_>,
    prompt: &str,
) -> Result<(), String> {
    let mut client = build_agent(
        params.shared,
        params.workflow,
        params.shared.acp_stdout_markdown_enabled(),
    );
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    client.prompts_log_run_dir = Some(params.artifacts.run_dir.clone());
    let timing = client.attach_run_timing_for_session();
    client
        .begin_coder_session(&params.artifacts.work_dir)
        .await
        .map_err(|e| e.to_string())?;
    timing
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .set_implement_display_name("summary");
    let run_res = run_summarize_coder_prompt(&mut client, params.artifacts, prompt).await;
    let end_res = client.end_coder_session().await.map_err(|e| e.to_string());
    let merged = crate::acp_post_run::prefer_primary_over_secondary(
        run_res,
        end_res,
        "end coder session",
    );
    crate::acp_post_run::emit_run_timing_json_only_after_acp(
        &mut client,
        &params.artifacts.run_dir,
        &timing,
        merged,
    )
}

/// Runs one summarize agent session when [`outer_loop_summarize_warranted`] is true.
pub(crate) async fn run_outer_loop_summarize_if_warranted(
    params: &OuterLoopSummarizeParams<'_>,
) -> Result<(), String> {
    if !params.agent_ran || !outer_loop_summarize_warranted(params.max_loops) {
        return Ok(());
    }
    let session_dotfile_backups =
        SessionDotfileBackups::snapshot(&params.artifacts.work_dir).map_err(|e| e.to_string())?;
    let prompt = render_kpop_summarize_prompt(
        params.store,
        params.artifacts,
        params.malvin_command,
        params.max_loops,
    )?;
    let acp_res = run_summarize_agent_session(params, &prompt).await;
    crate::acp_post_run::merge_acp_with_workspace_session_restore_and_check_abort(
        acp_res,
        &params.artifacts.work_dir,
        &session_dotfile_backups,
        &params.artifacts.artifact_result_md(),
    )
}
