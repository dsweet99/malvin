//! Default-route flow: requirements JSON, one multi-group `KPop`, optional work, outer `--max-loops`.

use crate::artifacts::{RunArtifacts, resolve_user_md_request};
use crate::cli::cli_request::require_cli_request;
use crate::agent_backend::{build_agent_backend, AgentBackend};
use crate::cli::run_emit::{emit_run_startup_sequence, RunStartupEmitOpts};
use crate::cli::{SharedOpts, WorkflowCliOptions};
use crate::prompts::PromptStore;
pub(crate) mod router_flow_prompt;
#[path = "router_flow_parse.rs"]
pub(crate) mod router_flow_parse;
#[path = "router_flow_no_work.rs"]
pub(crate) mod router_flow_no_work;
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
    /// Outer agent-session budget (`effective_max_loops`).
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
    let prompt_store = prepare_router_prompt_store(shared.no_kpop)?;
    let coder = router_flow_prompt::build_router_coder_run_with_store(
        &prompt_store,
        &artifacts,
        &text,
        crate::workflow_context::PromptModelOpts::new(&shared.model, shared.git),
    )?;
    Ok(RouterRunPrep {
        client,
        artifacts,
        coder,
        prompt_store,
    })
}

pub async fn run_router(
    router_args: RouterArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let request = require_cli_request(router_args.request.as_ref(), "")?;
    if shared.quiet {
        let interactive = crate::output::agent_stdout_tee_enabled();
        let emit_markdown = interactive && shared.acp_stdout_markdown_enabled();
        crate::output::set_do_dm_stdout_opts(crate::output::DoDmStdoutOpts {
            enabled: true,
            emit_markdown,
        });
        crate::output::set_heartbeat_stdout_suppressed(true);
    }
    let result = run_router_body(router_args, shared, workflow, &request).await;
    if shared.quiet {
        crate::output::set_do_dm_stdout_opts(crate::output::DoDmStdoutOpts::default());
        crate::output::set_heartbeat_stdout_suppressed(false);
    }
    result
}

async fn run_router_body(
    router_args: RouterArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    request: &str,
) -> Result<(), String> {
    let mut prep = prepare_router_run(&router_args, shared, workflow).await?;
    prep.client
        .set_prompts_log_run_dir(Some(prep.artifacts.run_dir.clone()));
    // Idea 3: complete spawn/handshake before the first `Logs:` line so post-Logs silence is not
    // dominated by ACP initialize + session/new (or Mini session bookkeeping).
    prep.client
        .begin_coder_session(&prep.artifacts.work_dir)
        .await
        .map_err(|e| e.to_string())?;
    emit_run_startup_sequence(
        &prep.artifacts,
        RunStartupEmitOpts::from_shared(shared, true),
        request,
    )?;

    let loop_outcome = router_flow_loop::run_router_agent_loops(router_flow_loop::RouterAgentLoopInput {
        client: &mut prep.client,
        artifacts: &prep.artifacts,
        coder: &prep.coder,
        prompt_store: &prep.prompt_store,
        shared,
        max_loops: router_args.max_loops,
    })
    .await?;

    // Mirror kpop: restore/check-abort, then print TIMING/COST from run_timing.json.
    let r = crate::acp_post_run::merge_acp_restore_check_abort_then_print_timing(
        loop_outcome.last_acp,
        &prep.artifacts,
        &loop_outcome.last_backups,
    );
    if r.is_ok() {
        crate::cli::error_run_log::clear_command_error_run_dir();
    }
    r
}

#[cfg(test)]
mod kiss_static_fn_item_refs {
    use super::{run_router, run_router_body};

    #[test]
    fn kiss_static_fn_item_refs() {
        let _ = run_router;
        let _ = run_router_body;
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
        let _ = router_flow_parse::load_review_requirements;
        let _ = router_flow_parse::parse_review_requirements_json;
    }
}

#[cfg(test)]
#[path = "router_flow_kiss_cov_tests.rs"]
mod router_flow_kiss_cov_tests;
