use std::collections::HashMap;
use std::path::PathBuf;

use malvin::acp::{AgentClient, AgentKpopMultiturnCtl};
use malvin::artifacts::{
    RunArtifacts, SessionDotfileBackups, create_kpop_run_artifacts, resolve_user_request,
};
use malvin::kpop_creative_enabled;
use malvin::kpop_multiturn_prompts::KpopMultiturnPrompts;
use malvin::kpop_progression::KpopMultiturnState;
use malvin::orchestrator::workflow_context_paths_only;
use malvin::output::{MALVIN_WHO, print_stdout_line};
use malvin::prompts::{PromptError, PromptStore};

use super::KpopArgs;
use super::LEARN_MIN_ELAPSED_MS;
use super::WorkflowCliOptions;
use super::build_agent;
use super::emit_run_startup_sequence;
use super::prepare_kpop_prompt_store;
use super::shared_opts::SharedOpts;
use super::timing_merge;
use super::timing_merge::emit_run_timing_after_acp;

pub(in crate::cli) fn kpop_prompt_store(
    kpop: &KpopArgs,
    workflow: WorkflowCliOptions,
) -> Result<PromptStore, String> {
    let needs_mbc2 = kpop_creative_enabled(kpop.p_creative);
    prepare_kpop_prompt_store(workflow, needs_mbc2)
}

pub struct KpopPrepared {
    pub(super) artifacts: RunArtifacts,
    pub(super) exp_log_path: PathBuf,
    pub(super) context: HashMap<String, String>,
    pub(super) text: String,
    pub(super) session_dotfile_backups: SessionDotfileBackups,
}

impl KpopPrepared {
    pub(in crate::cli) fn into_bug_followup_artifacts(
        self,
        plan_body: &str,
    ) -> Result<RunArtifacts, String> {
        let Self { mut artifacts, .. } = self;
        let plan_path = artifacts.run_dir.join("plan.md");
        std::fs::write(&plan_path, plan_body).map_err(|e| e.to_string())?;
        artifacts.plan_path = plan_path;
        Ok(artifacts)
    }
}

fn init_kpop_exp_log_file(artifacts: &RunArtifacts) -> Result<PathBuf, String> {
    let exp_log_path = artifacts.exp_log_path();
    let exp_parent = exp_log_path
        .parent()
        .ok_or_else(|| "kpop exp log path has no parent directory".to_string())?;
    std::fs::create_dir_all(exp_parent).map_err(|e| e.to_string())?;
    std::fs::write(&exp_log_path, "").map_err(|e| e.to_string())?;
    Ok(exp_log_path)
}

pub(in crate::cli) fn prepare_kpop_run(kpop: &KpopArgs) -> Result<KpopPrepared, String> {
    let (text, work_dir) = resolve_user_request(&kpop.request)?;
    let artifacts =
        create_kpop_run_artifacts(&text, Some(work_dir.as_path())).map_err(|e| e.to_string())?;
    let exp_log_path = init_kpop_exp_log_file(&artifacts)?;
    let session_dotfile_backups =
        SessionDotfileBackups::snapshot(&artifacts.work_dir)?;
    let mut context = workflow_context_paths_only(&artifacts, "kpop");
    context.insert(
        "quality_gates".to_string(),
        malvin::repo_gates::prompt_quality_gates_markdown_ephemeral(&artifacts.work_dir)?,
    );
    Ok(KpopPrepared {
        artifacts,
        exp_log_path,
        context,
        text,
        session_dotfile_backups,
    })
}

fn kpop_boot_store_client_prepared(
    kpop: &KpopArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(PromptStore, AgentClient, KpopPrepared), String> {
    let store = kpop_prompt_store(kpop, workflow)?;
    let emit_stdout_markdown = shared.acp_stdout_markdown_enabled();
    let mut client = build_agent(shared, workflow, emit_stdout_markdown);
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    let prepared = prepare_kpop_run(kpop)?;
    client.prompts_log_run_dir = Some(prepared.artifacts.run_dir.clone());
    super::error_run_log::set_command_error_run_dir(Some(prepared.artifacts.run_dir.clone()));
    Ok((store, client, prepared))
}

pub struct KpopAcpMultiturnCtx<'a> {
    pub client: &'a mut AgentClient,
    pub prepared: &'a KpopPrepared,
    pub workflow: WorkflowCliOptions,
    pub state: &'a mut KpopMultiturnState<'a>,
    pub store: &'a PromptStore,
}

pub(in crate::cli) async fn kpop_run_acp_multiturn(
    ctx: KpopAcpMultiturnCtx<'_>,
) -> Result<(), String> {
    let learn_owned = kpop_learn_bundle(
        ctx.store,
        &ctx.prepared.context,
        ctx.workflow.run_learn,
        &ctx.prepared.artifacts,
    )?;
    let timing = ctx.client.attach_run_timing_for_session();
    let acp_result = ctx
        .client
        .run_kpop_multiturn(AgentKpopMultiturnCtl {
            cwd: &ctx.prepared.artifacts.work_dir,
            kpop_log: ctx.prepared.artifacts.log_path("kpop"),
            learn: learn_owned,
            learn_min_elapsed_ms: LEARN_MIN_ELAPSED_MS,
            state: ctx.state,
            session_dotfile_backups: &ctx.prepared.session_dotfile_backups,
        })
        .await
        .map_err(|e| e.0);
    emit_run_timing_after_acp(
        ctx.client,
        &ctx.prepared.artifacts.run_dir,
        &timing,
        acp_result,
    )
}

pub async fn run_kpop(
    kpop: KpopArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let (store, mut client, prepared) = kpop_boot_store_client_prepared(&kpop, shared, workflow)?;

    kpop_emit_startup(&kpop, shared, &prepared.artifacts)?;

    let builder = KpopMultiturnPrompts::Turn(KpopTurnPrompts {
        store: &store,
        base: &prepared.context,
        request_text: &prepared.text,
        prepend_rules_once: true,
    });
    let mut state = KpopMultiturnState::new(
        builder,
        prepared.exp_log_path.clone(),
        kpop.max_hypotheses,
        kpop.p_creative,
    )?;

    let acp_result = kpop_run_acp_multiturn(KpopAcpMultiturnCtx {
        client: &mut client,
        prepared: &prepared,
        workflow,
        state: &mut state,
        store: &store,
    })
    .await;

    let r = timing_merge::merge_acp_with_workspace_session_restore_and_check_abort(
        acp_result,
        &prepared.artifacts.work_dir,
        &prepared.session_dotfile_backups,
        &prepared.artifacts.artifact_result_md(),
    );
    if r.is_ok() {
        super::error_run_log::clear_command_error_run_dir();
    }
    r?;
    print_stdout_line(MALVIN_WHO, "DONE");
    Ok(())
}
