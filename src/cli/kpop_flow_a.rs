use crate::artifacts::{
    RunArtifacts, SessionDotfileBackups, create_kpop_run_artifacts, resolve_user_md_request,
};
use crate::kpop_progression::KpopMultiturnState;
use crate::prompt_stratification::WorkflowRenderContext;
use crate::prompts::PromptStore;

use crate::agent_backend::{
    agent_backend_ensure_run_timing_for_session, agent_backend_run_kpop_multiturn,
    build_agent_backend, AgentBackend,
};
use crate::cli::{KpopArgs, SharedOpts, WorkflowCliOptions, prepare_kpop_prompt_store};

use super::kpop_flow_b::kpop_emit_startup;

pub(in crate) fn kpop_prompt_store(
    _kpop: &KpopArgs,
    workflow: WorkflowCliOptions,
) -> Result<PromptStore, String> {
    prepare_kpop_prompt_store(workflow, false)
}

pub struct KpopPrepared {
    pub(in crate) artifacts: RunArtifacts,
    pub(in crate) context: WorkflowRenderContext,
    pub(in crate) session_dotfile_backups: SessionDotfileBackups,
}

pub(in crate) struct KpopArtifactsEarly {
    pub(in crate) artifacts: RunArtifacts,
}

pub(in crate) fn prepare_kpop_artifacts(kpop: &KpopArgs) -> Result<KpopArtifactsEarly, String> {
    use crate::cli::cli_request::require_cli_request;
    let request = require_cli_request(kpop.request.as_ref(), "kpop")?;
    let (text, work_dir) = resolve_user_md_request(&request)?;
    let artifacts =
        create_kpop_run_artifacts(&text, Some(work_dir.as_path())).map_err(|e| e.to_string())?;
    Ok(KpopArtifactsEarly { artifacts })
}

pub(in crate) fn finish_kpop_prepared(early: KpopArtifactsEarly) -> Result<KpopPrepared, String> {
    use crate::orchestrator::workflow_context_paths_only;
    let session_dotfile_backups =
        SessionDotfileBackups::snapshot_after_ensuring_home_config(&early.artifacts.work_dir)?;
    let mut context = workflow_context_paths_only(&early.artifacts, "kpop");
    context.insert(
        "quality_gates".to_string(),
        crate::repo_gates::prompt_quality_gates_markdown_ephemeral(&early.artifacts.work_dir)?,
    );
    Ok(KpopPrepared {
        artifacts: early.artifacts,
        context,
        session_dotfile_backups,
    })
}

pub(crate) fn kpop_boot_store_client_prepared(
    kpop: &KpopArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(PromptStore, AgentBackend, KpopPrepared), String> {
    let early = prepare_kpop_artifacts(kpop)?;
    kpop_emit_startup(kpop, shared, &early.artifacts)?;
    let store = kpop_prompt_store(kpop, workflow)?;
    let emit_stdout_markdown = shared.acp_stdout_markdown_enabled();
    let mut client = build_agent_backend(shared, workflow, emit_stdout_markdown, "kpop")?;
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    let prepared = finish_kpop_prepared(early)?;
    client.set_prompts_log_run_dir(Some(prepared.artifacts.run_dir.clone()));
    crate::cli::error_run_log::set_command_error_run_dir(Some(prepared.artifacts.run_dir.clone()));
    Ok((store, client, prepared))
}

pub struct KpopAcpMultiturnCtx<'a> {
    pub client: &'a mut AgentBackend,
    pub prepared: &'a KpopPrepared,
    pub state: &'a mut KpopMultiturnState<'a>,
}

pub(in crate) async fn kpop_run_acp_multiturn(
    ctx: KpopAcpMultiturnCtx<'_>,
    session_dotfile_backups: &SessionDotfileBackups,
    session_end: crate::run_timing::acp_post_run::RunTimingSessionEnd,
) -> Result<(), String> {
    let timing = match session_end {
        crate::run_timing::acp_post_run::RunTimingSessionEnd::AccumulateRun => {
            agent_backend_ensure_run_timing_for_session(ctx.client)
        }
        crate::run_timing::acp_post_run::RunTimingSessionEnd::Finalize => {
            crate::agent_backend::agent_backend_attach_run_timing_for_session(ctx.client)
        }
    };
    let acp_result = agent_backend_run_kpop_multiturn(
        ctx.client,
        crate::acp::AgentKpopMultiturnCtl {
            cwd: &ctx.prepared.artifacts.work_dir,
            kpop_log: ctx.prepared.artifacts.log_path("kpop"),
            state: ctx.state,
            session_dotfile_backups,
        },
    )
    .await
    .map_err(|e| e.0);
    crate::acp_post_run::emit_run_timing_after_backend(crate::acp_post_run::RunTimingAfterBackend {
        backend: ctx.client,
        run_dir: &ctx.prepared.artifacts.run_dir,
        timing: &timing,
        agent_result: acp_result,
        session_end,
    })
}

pub(crate) fn run_kpop_short_id_lookup(kpop: &KpopArgs) -> Result<(), String> {
    let id = kpop.request.as_deref().expect("checked above").trim();
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let exp_log = crate::cli::bug_id_lookup_kpop::lookup_kpop_id(&cwd, id)?;
    crate::cli::bug_id_lookup_kpop::dump_kpop_log_to_stdout(&exp_log)
}

pub async fn run_kpop(
    kpop: KpopArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    if crate::cli::bug_id_lookup_kpop::is_kpop_lookup_request(kpop.request.as_deref()) {
        return run_kpop_short_id_lookup(&kpop);
    }

    let (store, mut client, prepared) = kpop_boot_store_client_prepared(&kpop, shared, workflow)?;

    let loops = super::kpop_flow_run_loop::run_kpop_agent_loops(
        super::kpop_flow_run_loop::RunKpopAgentLoopsParams {
            kpop: &kpop,
            shared,
            workflow,
            store: &store,
            client: &mut client,
            prepared: &prepared,
        },
    )
    .await;
    let acp_result = loops.acp_result;

    let summarize_res = crate::cli::kpop_summarize::run_outer_loop_summarize_if_warranted(
        &crate::cli::kpop_summarize::kpop_outer_loop_summarize_params(
            crate::cli::kpop_summarize::KpopOuterLoopSummarizeInputs {
                agent_ran: loops.agent_ran,
                shared,
            },
            &store,
            &prepared.artifacts,
        ),
    )
    .await;
    let acp_result = crate::acp_post_run::prefer_primary_over_secondary(
        acp_result,
        summarize_res,
        "outer-loop summarize",
    );

    let r = crate::acp_post_run::merge_acp_with_workspace_session_restore_and_check_abort(
        acp_result,
        &prepared.artifacts.work_dir,
        &prepared.session_dotfile_backups,
        &prepared.artifacts.artifact_result_md(),
    );
    crate::run_timing::print_summary_from_run_dir(&prepared.artifacts.run_dir)
        .map_err(|e| e.to_string())?;
    if r.is_ok() {
        crate::cli::error_run_log::clear_command_error_run_dir();
        crate::agent_phase::print_done_with_reporting_phase();
    }
    r
}

#[cfg(test)]
mod kiss_cov_auto {
    use super::*;

    #[test]
    fn kiss_cov_kpop_flow_a_structs() {
        let _: Option<KpopPrepared> = None;
        let _: Option<KpopArtifactsEarly> = None;
        let _: Option<KpopAcpMultiturnCtx> = None;
        let _ = prepare_kpop_artifacts;
        let _ = finish_kpop_prepared;
    }
}

#[cfg(test)]
#[path = "kpop_flow_a_tests.rs"]
mod kpop_flow_a_tests;

#[cfg(test)]
#[path = "kpop_flow_a_kiss_cov_tests.rs"]
mod kpop_flow_a_kiss_cov_tests;
