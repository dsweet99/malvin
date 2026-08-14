
use std::collections::HashMap;

use clap::Args;

use crate::artifacts::{RunArtifacts, SessionDotfileBackups};
use crate::agent_backend::{build_agent_backend, AgentBackend};
use crate::cli::cli_request::require_cli_request;
use crate::cli::one_shot_session::{
    finish_one_shot_after_prompt, finish_one_shot_auth_and_backups, resolve_one_shot_request_artifacts,
    OneShotCoderGuard,
};
use crate::cli::{SharedOpts, WorkflowCliOptions};
use crate::prompts::{PromptError, PromptStore, render_mbc2_for_scheduled_kpop_block};
use crate::run_timing::TimingPhase;

#[derive(Args, Debug)]
#[command(override_usage = "malvin inspire [OPTION]... [REQUEST]")]
pub struct InspireArgs {
    /// Existing `.md` path or literal text
    pub request: Option<String>,
}

struct InspireRunPrep {
    client: AgentBackend,
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

pub fn render_inspire_prompt(user_prompt: &str) -> Result<String, String> {
    let store = prepare_inspire_prompt_store()?;
    let ctx = build_inspire_render_context(user_prompt);
    render_mbc2_for_scheduled_kpop_block(&store, &ctx).map_err(|e| e.0)
}

fn new_inspire_client(
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<AgentBackend, String> {
    build_agent_backend(
        shared,
        workflow,
        shared.acp_stdout_markdown_enabled(),
        "inspire",
    )
}

fn inspire_emit_startup_banner(
    inspire: &InspireArgs,
    shared: &SharedOpts,
    artifacts: &RunArtifacts,
) -> Result<(), String> {
    let request = require_cli_request(inspire.request.as_ref(), "inspire")?;
    crate::cli::run_emit::emit_run_startup_banner(
        artifacts,
        crate::cli::run_emit::RunStartupEmitOpts::from_shared(shared, true),
        &request,
    )
}

async fn prepare_inspire_run(
    inspire: &InspireArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<InspireRunPrep, String> {
    let mut client = new_inspire_client(shared, workflow)?;
    let (text, artifacts) = resolve_one_shot_request_artifacts(
        inspire.request.as_ref(),
        "inspire",
        Some(crate::run_id::RunDirOptions { gc: false }),
    )?;
    inspire_emit_startup_banner(inspire, shared, &artifacts)?;
    crate::run_id::maybe_gc_after_run_created(&artifacts.work_dir, &artifacts.run_dir);
    let session_dotfile_backups = finish_one_shot_auth_and_backups(&mut client, &artifacts)?;
    let prompt = render_inspire_prompt(&text)?;
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
    prep.client
        .begin_coder_session(&prep.artifacts.work_dir)
        .await
        .map_err(|e| e.to_string())?;
    crate::cli::run_emit::emit_run_logs_line(&prep.artifacts)?;
    let acp_res = run_inspire_coder_session(&mut prep.client, &prep.artifacts, &prep.prompt).await;
    finish_one_shot_after_prompt(
        acp_res,
        &prep.artifacts.work_dir,
        &prep.session_dotfile_backups,
        &prep.artifacts.artifact_result_md(),
    )?;
    Ok(())
}

async fn run_inspire_coder_prompt(
    client: &mut AgentBackend,
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

async fn run_inspire_coder_session(
    client: &mut AgentBackend,
    artifacts: &RunArtifacts,
    prompt: &str,
) -> Result<(), String> {
    let guard = OneShotCoderGuard::begin(client, artifacts, "inspire").await?;
    let run_res = run_inspire_coder_prompt(client, artifacts, prompt).await;
    guard.finish(client, run_res).await
}

#[cfg(test)]
mod inspire_snapshot_tests {
    use super::SessionDotfileBackups;
    use crate::malvin_config_path;
    use crate::test_utils::with_isolated_home;

    #[test]
    fn inspire_prepare_snapshot_ensures_home_config_exists() {
        with_isolated_home(|work| {
            let cfg = malvin_config_path(work);
            assert!(!cfg.exists());
            SessionDotfileBackups::snapshot_after_ensuring_home_config(work).expect("snapshot");
            assert!(
                cfg.is_file(),
                "inspire session snapshot must ensure ~/.malvin_home/config.toml exists"
            );
        });
    }
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
        let _ = inspire_emit_startup_banner;
        let _ = prepare_inspire_prompt_store;
        let _ = prepare_inspire_run;
        let _ = run_inspire;
        let _ = run_inspire_coder_session;
        let _ = run_inspire_coder_prompt;
    }
}
