//! Post-session summarize agent when more than one `KPop` flow ran.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::artifacts::{RunArtifacts, SessionDotfileBackups};
use crate::agent_backend::{
    agent_backend_attach_run_timing_for_session, agent_backend_set_implement_display_name,
    build_agent_backend, AgentBackend,
};
use crate::cli::{SharedOpts, WorkflowCliOptions};
use crate::cli::workflow_kpop_shared::kpop_workflow_context;
use crate::prompts::{render_header, PromptError, PromptStore};
use crate::run_timing::TimingPhase;

const SUMMARIZE_PROMPT: &str = "kpop_summarize.md";

/// Inputs for [`run_outer_loop_summarize_if_warranted`].
pub(crate) struct OuterLoopSummarizeParams<'a> {
    pub agent_ran: bool,
    pub shared: &'a SharedOpts,
    pub workflow: WorkflowCliOptions,
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub malvin_command: &'a str,
}

/// Inputs for [`code_outer_loop_summarize_params`].
pub(crate) struct CodeOuterLoopSummarizeInputs<'a> {
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
        agent_ran: inputs.agent_ran,
        shared: inputs.shared,
        workflow: WorkflowCliOptions { force: false },
        store,
        artifacts,
        malvin_command: "malvin kpop",
    }
}

/// True when an exp log file exists and has content from an outer-loop agent session.
pub(crate) fn exp_log_has_flow_content(path: &Path) -> bool {
    std::fs::read(path)
        .ok()
        .is_some_and(|bytes| !bytes.is_empty())
}

/// Count `KPop` flows that ran in this session (one non-empty exp log per outer-loop iteration).
#[must_use]
pub(crate) fn kpop_flows_ran(artifacts: &RunArtifacts) -> usize {
    list_written_exp_logs(&artifacts.run_dir).len()
}

/// Whether an outer-loop summarize agent should run after `KPop` sessions complete.
#[must_use]
pub(crate) const fn outer_loop_summarize_warranted(kpop_flows_ran: usize) -> bool {
    kpop_flows_ran > 1
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
        .filter(|p| is_written_exp_log_path(p) && exp_log_has_flow_content(p))
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
    kpop_flows_ran: usize,
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
        kpop_flows_ran.to_string(),
    );
}

pub(crate) fn render_kpop_summarize_prompt(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    malvin_command: &str,
) -> Result<String, String> {
    let mut ctx = kpop_workflow_context(artifacts, malvin_command)?;
    insert_summarize_log_context(&mut ctx, artifacts, kpop_flows_ran(artifacts));
    let header = render_header(store, &ctx).map_err(|e: PromptError| e.0)?;
    let body = store
        .render_prompt_only(SUMMARIZE_PROMPT, &ctx)
        .map_err(|e: PromptError| e.0)?;
    Ok(format!("{}\n\n{}", header.trim_end(), body.trim_end()))
}

pub(crate) async fn run_summarize_coder_prompt(
    client: &mut AgentBackend,
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
    let mut client = build_agent_backend(
        params.shared,
        params.workflow,
        params.shared.acp_stdout_markdown_enabled(),
        "kpop",
    )?;
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    client
        .set_prompts_log_run_dir(Some(params.artifacts.run_dir.clone()));
    let timing = agent_backend_attach_run_timing_for_session(&mut client);
    client
        .begin_coder_session(&params.artifacts.work_dir)
        .await
        .map_err(|e| e.to_string())?;
    agent_backend_set_implement_display_name(&client, "summary");
    let run_res = run_summarize_coder_prompt(&mut client, params.artifacts, prompt).await;
    let end_res = client.end_coder_session().await.map_err(|e| e.to_string());
    let merged = crate::acp_post_run::prefer_primary_over_secondary(
        run_res,
        end_res,
        "end coder session",
    );
    crate::acp_post_run::emit_run_timing_json_only_after_backend(
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
    let flows_ran = kpop_flows_ran(params.artifacts);
    if !params.agent_ran || !outer_loop_summarize_warranted(flows_ran) {
        return Ok(());
    }
    let session_dotfile_backups =
        SessionDotfileBackups::snapshot(&params.artifacts.work_dir).map_err(|e| e.to_string())?;
    let prompt = render_kpop_summarize_prompt(
        params.store,
        params.artifacts,
        params.malvin_command,
    )?;
    let acp_res = run_summarize_agent_session(params, &prompt).await;
    crate::acp_post_run::merge_acp_with_workspace_session_restore_and_check_abort(
        acp_res,
        &params.artifacts.work_dir,
        &session_dotfile_backups,
        &params.artifacts.artifact_result_md(),
    )
}
