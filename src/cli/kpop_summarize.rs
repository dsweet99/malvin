
#[path = "kpop_summarize_inline.rs"]
mod kpop_summarize_inline;
pub(crate) use kpop_summarize_inline::{
    maybe_run_gate_inline_summarize, GateInlineSummarizeCtx,
};

use crate::prompt_stratification::{join_labeled_strata, PromptStratum, WorkflowRenderContext};
use std::path::{Path, PathBuf};

use crate::artifacts::RunArtifacts;
use crate::agent_backend::{agent_backend_set_implement_display_name, AgentBackend};
use crate::prompts::{render_header, PromptError, PromptStore};
use crate::run_timing::TimingPhase;

const SUMMARIZE_PROMPT: &str = "kpop_summarize.md";

pub(crate) fn exp_log_has_flow_content(path: &Path) -> bool {
    std::fs::read(path)
        .ok()
        .is_some_and(|bytes| !bytes.is_empty())
}

#[must_use]
pub(crate) fn kpop_flows_ran(artifacts: &RunArtifacts) -> usize {
    list_written_exp_logs(&artifacts.run_dir).len()
}

#[must_use]
#[cfg(test)]
pub(crate) const fn outer_loop_summarize_warranted(kpop_flows_ran: usize) -> bool {
    kpop_flows_ran > 1
}

#[must_use]
pub(crate) const fn should_inline_outer_loop_summarize_on_gate_iteration(
    iteration: usize,
    total_iterations: usize,
) -> bool {
    if iteration < 2 {
        return false;
    }
    iteration == total_iterations
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
    ctx: &mut WorkflowRenderContext,
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
    model: &str,
    git: bool,
) -> Result<String, String> {
    let mut ctx =
        crate::cli::workflow_kpop_shared::kpop_workflow_context_without_gates(artifacts, model, git)?;
    insert_summarize_log_context(&mut ctx, artifacts, kpop_flows_ran(artifacts));
    let header = render_header(store, ctx.as_map()).map_err(|e: PromptError| e.0)?;
    let body = store
        .render_prompt_only(SUMMARIZE_PROMPT, ctx.as_map())
        .map_err(|e: PromptError| e.0)?;
    Ok(join_labeled_strata([
        (PromptStratum::WorkflowHeader, header),
        (PromptStratum::GateLoopBlock, body),
    ]))
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

pub(crate) async fn run_inline_summarize_coder_prompt(
    client: &mut AgentBackend,
    store: &PromptStore,
    artifacts: &RunArtifacts,
    opts: crate::workflow_context::PromptModelOpts<'_>,
) -> Result<(), String> {
    agent_backend_set_implement_display_name(client, "summary");
    let prompt = render_kpop_summarize_prompt(store, artifacts, opts.model, opts.git)?;
    run_summarize_coder_prompt(client, artifacts, &prompt).await
}

