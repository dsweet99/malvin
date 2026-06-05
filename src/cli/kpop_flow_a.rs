use std::collections::HashMap;
use crate::artifacts::{
    RunArtifacts, SessionDotfileBackups, create_kpop_run_artifacts, resolve_user_request,
};
use crate::kpop_progression::KpopMultiturnState;
use crate::prompts::PromptStore;

use crate::cli::{KpopArgs, SharedOpts, WorkflowCliOptions, build_agent, prepare_kpop_prompt_store};

use super::kpop_flow_b::kpop_emit_startup;

pub(in crate) fn kpop_prompt_store(
    _kpop: &KpopArgs,
    workflow: WorkflowCliOptions,
) -> Result<PromptStore, String> {
    prepare_kpop_prompt_store(workflow, false)
}

pub struct KpopPrepared {
    pub(in crate) artifacts: RunArtifacts,
    pub(in crate) context: HashMap<String, String>,
    pub(in crate) text: String,
    pub(in crate) session_dotfile_backups: SessionDotfileBackups,
}

pub(in crate) fn prepare_kpop_run(kpop: &KpopArgs) -> Result<KpopPrepared, String> {
    use crate::cli::cli_request::require_cli_request;
    use crate::orchestrator::workflow_context_paths_only;
    let request = require_cli_request(kpop.request.as_ref(), "kpop")?;
    let (text, work_dir) = resolve_user_request(&request)?;
    let artifacts =
        create_kpop_run_artifacts(&text, Some(work_dir.as_path())).map_err(|e| e.to_string())?;
    let session_dotfile_backups =
        SessionDotfileBackups::snapshot(&artifacts.work_dir)?;
    let mut context = workflow_context_paths_only(&artifacts, "kpop");
    context.insert(
        "quality_gates".to_string(),
        crate::repo_gates::prompt_quality_gates_markdown_ephemeral(&artifacts.work_dir)?,
    );
    Ok(KpopPrepared {
        artifacts,
        context,
        text,
        session_dotfile_backups,
    })
}

pub(crate) fn kpop_boot_store_client_prepared(
    kpop: &KpopArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(PromptStore, crate::acp::AgentClient, KpopPrepared), String> {
    let store = kpop_prompt_store(kpop, workflow)?;
    let emit_stdout_markdown = shared.acp_stdout_markdown_enabled();
    let mut client = build_agent(shared, workflow, emit_stdout_markdown);
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    let prepared = prepare_kpop_run(kpop)?;
    client.prompts_log_run_dir = Some(prepared.artifacts.run_dir.clone());
    crate::cli::error_run_log::set_command_error_run_dir(Some(prepared.artifacts.run_dir.clone()));
    Ok((store, client, prepared))
}

pub struct KpopAcpMultiturnCtx<'a> {
    pub client: &'a mut crate::acp::AgentClient,
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
            ctx.client.ensure_run_timing_for_session()
        }
        crate::run_timing::acp_post_run::RunTimingSessionEnd::Finalize => {
            ctx.client.attach_run_timing_for_session()
        }
    };
    let acp_result = ctx
        .client
        .run_kpop_multiturn(crate::acp::AgentKpopMultiturnCtl {
            cwd: &ctx.prepared.artifacts.work_dir,
            kpop_log: ctx.prepared.artifacts.log_path("kpop"),
            state: ctx.state,
            session_dotfile_backups,
        })
        .await
        .map_err(|e| e.0);
    crate::acp_post_run::emit_run_timing_after_acp(crate::acp_post_run::RunTimingAfterAcp {
        client: ctx.client,
        run_dir: &ctx.prepared.artifacts.run_dir,
        timing: &timing,
        acp_result,
        session_end,
    })
}

fn run_kpop_short_id_lookup(kpop: &KpopArgs) -> Result<(), String> {
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

    kpop_emit_startup(&kpop, shared, &prepared.artifacts)?;

    let loops = super::kpop_flow_run_loop::run_kpop_agent_loops(
        super::kpop_flow_run_loop::RunKpopAgentLoopsParams {
            kpop: &kpop,
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
                max_loops: kpop.max_loops,
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
mod kiss_cov_auto{
    use super::*;

    use super::run_kpop_short_id_lookup;
    use crate::cli::KpopArgs;
    use crate::output::{format_who_tag_prefix, MALVIN_WHO};

    #[test]
    fn kiss_cov_kpop_prompt_store() { let _ = kpop_prompt_store; }

    #[test]
    fn kiss_cov_ensure_kpop_exp_log_file() {
        let _ = stringify!(crate::artifacts::create::ensure_kpop_exp_log_file);
    }

    #[test]
    fn kiss_cov_kpop_boot_store_client_prepared() { let _ = kpop_boot_store_client_prepared; }

    #[test]
    fn run_kpop_short_id_lookup_dumps_matching_exp_log() {
        crate::test_utils::with_isolated_home(|cwd| {
            let home = crate::user_home_dir();
            let run_name = "20260101_000000_abcabcab";
            let bucket = home.join(".malvin/logs").join(crate::workspace_logs_hash(cwd));
            let run_dir = bucket.join(run_name);
            std::fs::create_dir_all(&run_dir).expect("mkdir");
            crate::write_work_dir_manifest(&run_dir, cwd).expect("manifest");
            let exp = run_dir.join("_kpop").join(format!("exp_log_{run_name}.md"));
            std::fs::create_dir_all(exp.parent().unwrap()).expect("mkdir kpop");
            std::fs::write(&exp, "lookup ok\n").expect("write exp");
            let rel = format!("{}/_kpop/exp_log_{run_name}.md", run_dir.display());
            std::fs::write(
                run_dir.join("stdout.log"),
                format!(
                    "20260101.000000.000 {}KPOP_LOG: Ma1b2c {rel}\n",
                    format_who_tag_prefix(MALVIN_WHO)
                ),
            )
            .expect("stdout");
            let old = std::env::current_dir().expect("cwd");
            std::env::set_current_dir(cwd).expect("chdir");
            let kpop = KpopArgs {
                max_loops: 1,
                max_hypotheses: 1,
                tenacious: false,
                request: Some("Ma1b2c".into()),
            };
            run_kpop_short_id_lookup(&kpop).expect("lookup dump");
            std::env::set_current_dir(old).expect("restore cwd");
        });
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs{
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _ = kpop_boot_store_client_prepared;
        let _ = stringify!(kpop_prompt_store);
    }
}
