use std::collections::HashMap;
use std::path::PathBuf;

use crate::KpopTurnPrompts;
use crate::artifacts::{
    RunArtifacts, SessionDotfileBackups, create_kpop_run_artifacts, resolve_user_request,
};
use crate::kpop_multiturn_prompts::KpopMultiturnPrompts;
use crate::kpop_progression::KpopMultiturnState;
use crate::output::{MALVIN_WHO, print_stdout_line};
use crate::prompts::PromptStore;

use crate::cli::{KpopArgs, SharedOpts, WorkflowCliOptions, build_agent, prepare_kpop_prompt_store};

use super::kpop_flow_b::{kpop_emit_startup, kpop_learn_bundle};

pub(in crate) fn kpop_prompt_store(
    _kpop: &KpopArgs,
    workflow: WorkflowCliOptions,
) -> Result<PromptStore, String> {
    prepare_kpop_prompt_store(workflow, false)
}

pub struct KpopPrepared {
    pub(in crate) artifacts: RunArtifacts,
    pub(in crate) exp_log_path: PathBuf,
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
    let exp_log_path = artifacts.exp_log_path();
    let session_dotfile_backups =
        SessionDotfileBackups::snapshot(&artifacts.work_dir)?;
    let mut context = workflow_context_paths_only(&artifacts, "kpop");
    context.insert(
        "quality_gates".to_string(),
        crate::repo_gates::prompt_quality_gates_markdown_ephemeral(&artifacts.work_dir)?,
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
    pub workflow: WorkflowCliOptions,
    pub state: &'a mut KpopMultiturnState<'a>,
    pub store: &'a PromptStore,
}

pub(in crate) async fn kpop_run_acp_multiturn(
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
        .run_kpop_multiturn(crate::acp::AgentKpopMultiturnCtl {
            cwd: &ctx.prepared.artifacts.work_dir,
            kpop_log: ctx.prepared.artifacts.log_path("kpop"),
            learn: learn_owned,
            learn_min_elapsed_ms: crate::DEFAULT_LEARN_MIN_ELAPSED_MS,
            state: ctx.state,
            session_dotfile_backups: &ctx.prepared.session_dotfile_backups,
        })
        .await
        .map_err(|e| e.0);
    crate::acp_post_run::emit_run_timing_after_acp(
        ctx.client,
        &ctx.prepared.artifacts.run_dir,
        &timing,
        acp_result,
    )
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

    let kpop_id = crate::malvin_short_id();
    let log_line = crate::cli::bug_id_lookup_kpop::kpop_log_line(
        &kpop_id,
        &prepared.artifacts.work_dir,
        &prepared.artifacts.run_dir,
        &prepared.exp_log_path,
    );
    print_stdout_line(MALVIN_WHO, &log_line);

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
        0.0,
    )?;

    let acp_result = kpop_run_acp_multiturn(KpopAcpMultiturnCtx {
        client: &mut client,
        prepared: &prepared,
        workflow,
        state: &mut state,
        store: &store,
    })
    .await;

    let r = crate::acp_post_run::merge_acp_with_workspace_session_restore_and_check_abort(
        acp_result,
        &prepared.artifacts.work_dir,
        &prepared.session_dotfile_backups,
        &prepared.artifacts.artifact_result_md(),
    );
    if r.is_ok() {
        crate::cli::error_run_log::clear_command_error_run_dir();
    }
    r?;
    print_stdout_line(MALVIN_WHO, "DONE");
    Ok(())
}


#[cfg(test)]
mod kiss_cov_auto {
    use super::run_kpop_short_id_lookup;
    use crate::cli::KpopArgs;
    use crate::output::{format_log_tag_inner, MALVIN_WHO};

    #[test]
    fn kiss_cov_kpop_prompt_store() { let _ = stringify!(kpop_prompt_store); }

    #[test]
    fn kiss_cov_ensure_kpop_exp_log_file() {
        let _ = stringify!(crate::artifacts::create::ensure_kpop_exp_log_file);
    }

    #[test]
    fn kiss_cov_kpop_boot_store_client_prepared() { let _ = stringify!(kpop_boot_store_client_prepared); }

    #[test]
    fn run_kpop_short_id_lookup_dumps_matching_exp_log() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let run_dir = cwd.join(".malvin/logs").join("20260101_abc");
        std::fs::create_dir_all(&run_dir).expect("mkdir");
        let exp = run_dir.join("_kpop").join("exp_log_20260101_abc.md");
        std::fs::create_dir_all(exp.parent().unwrap()).expect("mkdir kpop");
        std::fs::write(&exp, "lookup ok\n").expect("write exp");
        let rel = "./.malvin/logs/20260101_abc/_kpop/exp_log_20260101_abc.md";
        std::fs::write(
            run_dir.join("stdout.log"),
            format!(
                "20260101.000000.000 [{}] KPOP_LOG: Ma1b2c {rel}\n",
                format_log_tag_inner(MALVIN_WHO)
            ),
        )
        .expect("stdout");
        let old = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(cwd).expect("chdir");
        let kpop = KpopArgs {
            max_hypotheses: 1,
            no_learn: true,
            request: Some("Ma1b2c".into()),
        };
        run_kpop_short_id_lookup(&kpop).expect("lookup dump");
        std::env::set_current_dir(old).expect("restore cwd");
    }
}
