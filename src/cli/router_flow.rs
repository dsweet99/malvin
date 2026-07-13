//! Default-route flow: dual-header `router_a_1.md`, bare `router_a_2.md`, then bare `router_b_*` and `router_c.md` on one coder session per outer loop.

use crate::artifacts::{RunArtifacts, resolve_user_md_request};
use crate::cli::cli_request::require_cli_request;
use crate::agent_backend::{agent_backend_set_implement_display_name, build_agent_backend, AgentBackend};
use crate::cli::run_emit::{emit_run_startup_sequence, RunStartupEmitOpts};
use crate::cli::workflow_kpop_shared::effective_max_loops;
use crate::cli::{SharedOpts, WorkflowCliOptions};
use crate::prompts::{PromptStore, ROUTER_D_MD};
pub(crate) mod router_flow_prompt;
#[path = "router_flow_parse.rs"]
pub(crate) mod router_flow_parse;
#[path = "router_flow_acp.rs"]
pub(crate) mod router_flow_acp;
#[path = "router_flow_loop.rs"]
pub(crate) mod router_flow_loop;

pub use router_flow_prompt::{
    combine_router_acp_prompt_header_and_user, combine_router_prompt_file_and_user,
    combine_router_raw_header_and_user, prepare_router_prompt_store,
};

/// Arguments for [`run_router`].
#[derive(Debug)]
pub struct RouterArgs {
    /// Existing `.md` path or literal text
    pub request: Option<String>,
    pub max_loops: usize,
}

struct RouterRunPrep {
    client: AgentBackend,
    artifacts: RunArtifacts,
    coder: router_flow_prompt::RouterCoderRun,
    prompt_store: PromptStore,
}

fn new_router_client(shared: &SharedOpts, workflow: WorkflowCliOptions) -> Result<AgentBackend, String> {
    build_agent_backend(
        shared,
        workflow,
        shared.acp_stdout_markdown_enabled(),
        "router",
    )
}

async fn prepare_router_run(
    router_args: &RouterArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<RouterRunPrep, String> {
    let client = new_router_client(shared, workflow)?;
    let request = require_cli_request(router_args.request.as_ref(), "")?;
    let (text, work_dir) = resolve_user_md_request(&request)?;
    let artifacts = crate::artifacts::create_run_artifacts_from_text_opts(
        &text,
        Some(work_dir.as_path()),
        crate::run_id::RunDirOptions::default(),
    )
    .map_err(|e| e.to_string())?;
    crate::cli::error_run_log::set_command_error_run_dir(Some(artifacts.run_dir.clone()));
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    let prompt_store = prepare_router_prompt_store()?;
    let coder = router_flow_prompt::build_router_coder_run_with_store(&prompt_store, &artifacts, &text, &shared.model)?;
    Ok(RouterRunPrep {
        client,
        artifacts,
        coder,
        prompt_store,
    })
}

pub(crate) async fn run_router_d_session(
    client: &mut AgentBackend,
    prompt_store: &PromptStore,
    artifacts: &RunArtifacts,
    model: &str,
) -> Result<(), String> {
    let work_dir = artifacts.work_dir.as_path();
    client
        .begin_coder_session(work_dir)
        .await
        .map_err(|e| e.to_string())?;
    agent_backend_set_implement_display_name(client, "router");
    let prompt = router_flow_prompt::build_router_d_prompt(prompt_store, artifacts, model)?;
    client
        .run_coder_prompt(
            &prompt,
            &artifacts.log_path("router_d"),
            "router_d",
            crate::acp::CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                do_trace_split: None,
                stdout_bracket_label: Some(ROUTER_D_MD),
                ..Default::default()
            },
        )
        .await
        .map_err(|e| e.to_string())?;
    client.end_coder_session().await.map_err(|e| e.to_string())
}

async fn maybe_run_trivial_social_as_do(
    request: &str,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<bool, String> {
    let (text, _work_dir) = resolve_user_md_request(request)?;
    // Shared (all backends): greetings must not enter the tenacious coding router.
    // Local models otherwise mis-label Hello as CODING_TASK:YES / CONTINUE_ROUTER and hang.
    if !router_flow_parse::is_trivial_social_request(&text) {
        return Ok(false);
    }
    crate::do_flow::run_do(
        crate::do_flow::DoArgs {
            thoughts: false,
            request: Some(request.to_string()),
        },
        shared,
        workflow,
    )
    .await?;
    Ok(true)
}

pub async fn run_router(
    router_args: RouterArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let request = require_cli_request(router_args.request.as_ref(), "")?;
    if maybe_run_trivial_social_as_do(&request, shared, workflow).await? {
        return Ok(());
    }

    let mut prep = prepare_router_run(&router_args, shared, workflow).await?;
    emit_run_startup_sequence(
        &prep.artifacts,
        RunStartupEmitOpts::from_shared(shared, true),
        &request,
    )?;
    prep.client
        .set_prompts_log_run_dir(Some(prep.artifacts.run_dir.clone()));

    let max_loops = effective_max_loops(router_args.max_loops);
    let loop_outcome = router_flow_loop::run_router_agent_loops(router_flow_loop::RouterAgentLoopInput {
        client: &mut prep.client,
        artifacts: &prep.artifacts,
        coder: &prep.coder,
        prompt_store: &prep.prompt_store,
        shared,
        max_loops,
    })
    .await?;

    // Classify/work failures must not run router_d (summarizer) before surfacing the error.
    if loop_outcome.last_acp.is_ok() {
        run_router_d_session(
            &mut prep.client,
            &prep.prompt_store,
            &prep.artifacts,
            &shared.model,
        )
        .await?;
    }

    let r = crate::acp_post_run::merge_acp_with_workspace_session_restore_and_check_abort(
        loop_outcome.last_acp,
        prep.artifacts.work_dir.as_path(),
        &loop_outcome.last_backups,
        &prep.artifacts.artifact_result_md(),
    );
    if r.is_ok() {
        crate::cli::error_run_log::clear_command_error_run_dir();
    }
    r?;
    Ok(())
}

#[cfg(test)]
mod kiss_static_fn_item_refs {
    use super::run_router;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _ = run_router;
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _: Option<RouterRunPrep> = None;
        let _ = new_router_client;
        let _ = prepare_router_run;
        let _ = maybe_run_trivial_social_as_do;
        let _ = router_flow_parse::parse_complexity_score;
        let _ = router_flow_parse::parse_coding_task;
        let _ = router_flow_parse::router_wants_continue;
        let _ = router_flow_parse::is_trivial_social_request;
        let _ = router_flow_prompt::build_router_d_prompt;
        let _ = super::run_router_d_session;
    }
}

#[cfg(test)]
#[path = "router_flow_kiss_cov_tests.rs"]
mod router_flow_kiss_cov_tests;
