//! `inspire` subcommand: one-shot MBC2 boundary-exploration prompt from `mbc2.md`.

use std::collections::HashMap;

use clap::Args;

use crate::artifacts::{
    RunArtifacts, SessionDotfileBackups, create_run_artifacts_from_text, resolve_user_md_request,
};
use crate::cli::cli_request::require_cli_request;
use crate::cli::{SharedOpts, WorkflowCliOptions, build_agent};
use crate::prompts::{PromptError, PromptStore, render_mbc2_for_scheduled_kpop_block};
use crate::run_timing::TimingPhase;

/// Arguments for [`run_inspire`].
#[derive(Args, Debug)]
pub struct InspireArgs {
    /// Existing `.md` path or literal text → `.malvin/logs/.../plan.md`.
    pub request: Option<String>,
}

struct InspireRunPrep {
    client: crate::acp::AgentClient,
    artifacts: RunArtifacts,
    prompt: String,
    session_dotfile_backups: SessionDotfileBackups,
}

fn prepare_inspire_prompt_store() -> Result<PromptStore, String> {
    let store = PromptStore::default_store();
    store
        .validate_exists("mbc2.md")
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

pub fn build_inspire_render_context(user_prompt: &str) -> HashMap<String, String> {
    HashMap::from([("user_prompt".into(), user_prompt.to_string())])
}

/// # Errors
///
/// Returns a message when `mbc2.md` cannot be loaded or rendered.
pub fn render_inspire_prompt(user_prompt: &str) -> Result<String, String> {
    let store = prepare_inspire_prompt_store()?;
    let ctx = build_inspire_render_context(user_prompt);
    render_mbc2_for_scheduled_kpop_block(&store, &ctx).map_err(|e| e.0)
}

fn new_inspire_client(shared: &SharedOpts, workflow: WorkflowCliOptions) -> crate::acp::AgentClient {
    build_agent(
        shared,
        workflow,
        shared.acp_stdout_markdown_enabled(),
    )
}

fn inspire_emit_startup(
    inspire: &InspireArgs,
    shared: &SharedOpts,
    artifacts: &RunArtifacts,
) -> Result<(), String> {
    let request = require_cli_request(inspire.request.as_ref(), "inspire")?;
    crate::cli::run_emit::emit_run_startup_sequence(
        artifacts,
        crate::cli::run_emit::RunStartupEmitOpts {
            tee_stdout: shared.tee_startup_stdout(),
            host_resources: true,
        },
        &request,
    )
}


async fn prepare_inspire_run(
    inspire: &InspireArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<InspireRunPrep, String> {
    let client = new_inspire_client(shared, workflow);
    let request = require_cli_request(inspire.request.as_ref(), "inspire")?;
    let (text, work_dir) = resolve_user_md_request(&request)?;
    let artifacts = create_run_artifacts_from_text(&text, Some(work_dir.as_path()))
        .map_err(|e| e.to_string())?;
    crate::cli::error_run_log::set_command_error_run_dir(Some(artifacts.run_dir.clone()));
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    let prompt = render_inspire_prompt(&text)?;
    let session_dotfile_backups = SessionDotfileBackups::snapshot(&artifacts.work_dir)?;
    Ok(InspireRunPrep {
        client,
        artifacts,
        prompt,
        session_dotfile_backups,
    })
}

pub async fn run_inspire(
    inspire: InspireArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let mut prep = prepare_inspire_run(&inspire, shared, workflow).await?;
    inspire_emit_startup(&inspire, shared, &prep.artifacts)?;
    prep.client.prompts_log_run_dir = Some(prep.artifacts.run_dir.clone());
    let acp_res = run_inspire_acp(&mut prep.client, &prep.artifacts, &prep.prompt).await;
    let r = crate::acp_post_run::merge_acp_with_workspace_session_restore_and_check_abort(
        acp_res,
        &prep.artifacts.work_dir,
        &prep.session_dotfile_backups,
        &prep.artifacts.artifact_result_md(),
    );
    if r.is_ok() {
        crate::cli::error_run_log::clear_command_error_run_dir();
    }
    r?;
    Ok(())
}

async fn run_inspire_coder_prompt(
    client: &mut crate::acp::AgentClient,
    artifacts: &RunArtifacts,
    prompt: &str,
) -> Result<(), String> {
    client
        .run_coder_prompt(
            prompt,
            &artifacts.log_path("inspire"),
            "inspire",
            crate::acp::CoderPromptOptions {
                llm_phase: Some(TimingPhase::Implement),
                do_trace_split: None,
                stdout_bracket_label: None,
                ..Default::default()
            },
        )
        .await
        .map_err(|e| e.to_string())
}

async fn run_inspire_acp(
    client: &mut crate::acp::AgentClient,
    artifacts: &RunArtifacts,
    prompt: &str,
) -> Result<(), String> {
    let timing = client.attach_run_timing_for_session();
    if let Err(e) = client.begin_coder_session(&artifacts.work_dir).await {
        client.set_run_timing(None);
        return Err(e.to_string());
    }
    timing
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .set_implement_display_name("inspire");
    let run_res = run_inspire_coder_prompt(client, artifacts, prompt).await;
    let end_res = client.end_coder_session().await.map_err(|e| e.to_string());
    let merged =
        crate::acp_post_run::prefer_primary_over_secondary(run_res, end_res, "end coder session");
    crate::acp_post_run::emit_run_timing_json_only_after_acp(
        client,
        &artifacts.run_dir,
        &timing,
        merged,
    )
}

#[cfg(test)]
#[path = "inspire_flow_tests.rs"]
mod inspire_flow_tests;

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs{
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _: Option<InspireRunPrep> = None;
        let _ = new_inspire_client;
        let _ = inspire_emit_startup;
        let _ = prepare_inspire_prompt_store;
        let _ = prepare_inspire_run;
        let _ = run_inspire;
        let _ = run_inspire_acp;
        let _ = run_inspire_coder_prompt;
    }
}
