//! KPOP subcommand: artifacts, prompt assembly, and ACP dispatch.

use std::collections::HashMap;
use std::path::PathBuf;

use malvin::acp::AgentClient;
use malvin::artifacts::{
    GroundingBackup, RunArtifacts, backup_workspace_grounding_if_present,
    create_kpop_run_artifacts, resolve_user_request,
};
use malvin::kpop_creative_enabled;
use malvin::kpop_multiturn::KpopMultiturnState;
use malvin::kpop_multiturn_prompts::KpopMultiturnPrompts;
use malvin::orchestrator::workflow_context_paths_only;
use malvin::output::{MALVIN_WHO, print_stdout_line};
use malvin::prompts::{PromptError, PromptStore, merged_coding_rules};

use super::KpopArgs;
use super::LEARN_MIN_ELAPSED_MS;
use super::WorkflowCliOptions;
use super::build_agent;
use super::emit_run_startup_sequence;
use super::prepare_kpop_prompt_store;
use super::repo_checks::{self, RepoGateOutput};
use super::shared_opts::SharedOpts;
use super::timing_merge::{emit_run_timing_after_acp, merge_acp_with_grounding_restore};

fn kpop_prompt_store(kpop: &KpopArgs, workflow: WorkflowCliOptions) -> Result<PromptStore, String> {
    let needs_mbc2 = kpop_creative_enabled(kpop.p_creative);
    prepare_kpop_prompt_store(workflow, needs_mbc2)
}

pub struct KpopTurnPrompts<'a> {
    store: &'a PromptStore,
    base: &'a HashMap<String, String>,
    request_text: &'a str,
}

impl KpopTurnPrompts<'_> {
    fn render_turn_with_body(
        &self,
        body_file: &str,
        ctx: &HashMap<String, String>,
    ) -> Result<String, String> {
        let common = self
            .store
            .render_prompt_only("kpop_common.md", ctx)
            .map_err(|e: PromptError| e.0)?;
        let body = self
            .store
            .render_prompt_only(body_file, ctx)
            .map_err(|e: PromptError| e.0)?;
        let rules = merged_coding_rules(self.store, ctx).map_err(|e: PromptError| e.0)?;
        Ok(format!(
            "{}\n\n{}\n\n{}",
            rules.trim_end(),
            common.trim_end(),
            body.trim_end()
        ))
    }
}

impl KpopMultiturnPrompts for KpopTurnPrompts<'_> {
    fn kpop_block(
        &mut self,
        want: usize,
        remaining_after_this_turn: usize,
    ) -> Result<String, String> {
        let mut ctx = self.base.clone();
        ctx.insert("want".to_string(), want.to_string());
        ctx.insert(
            "remaining_hypotheses".to_string(),
            remaining_after_this_turn.to_string(),
        );
        ctx.insert("user_request".to_string(), self.request_text.to_string());
        self.render_turn_with_body("kpop_block.md", &ctx)
    }

    fn mbc2_pure(&mut self) -> Result<String, String> {
        let mut ctx = self.base.clone();
        ctx.insert("user_request".to_string(), self.request_text.to_string());
        self.render_turn_with_body("mbc2_pure.md", &ctx)
    }
}

pub struct KpopPrepared {
    artifacts: RunArtifacts,
    exp_log_path: PathBuf,
    context: HashMap<String, String>,
    text: String,
    grounding_backup: GroundingBackup,
}

fn prepare_kpop_run(kpop: &KpopArgs) -> Result<KpopPrepared, String> {
    let (text, work_dir) = resolve_user_request(&kpop.request)?;
    let artifacts =
        create_kpop_run_artifacts(&text, Some(work_dir.as_path())).map_err(|e| e.to_string())?;
    let grounding_backup = backup_workspace_grounding_if_present(&artifacts.work_dir)?;
    let exp_log_path = artifacts.exp_log_path();
    let exp_parent = exp_log_path
        .parent()
        .ok_or_else(|| "kpop exp log path has no parent directory".to_string())?;
    std::fs::create_dir_all(exp_parent).map_err(|e| e.to_string())?;
    std::fs::write(&exp_log_path, "").map_err(|e| e.to_string())?;
    let context = workflow_context_paths_only(&artifacts);
    Ok(KpopPrepared {
        artifacts,
        exp_log_path,
        context,
        text,
        grounding_backup,
    })
}

pub struct KpopAcpMultiturnCtx<'a, 'b> {
    pub client: &'a mut AgentClient,
    pub prepared: &'a KpopPrepared,
    pub workflow: WorkflowCliOptions,
    pub state: &'a mut KpopMultiturnState<KpopTurnPrompts<'b>>,
    pub store: &'a PromptStore,
}

pub async fn kpop_run_acp_multiturn(ctx: KpopAcpMultiturnCtx<'_, '_>) -> Result<(), String> {
    let learn_stored = kpop_learn_bundle(
        ctx.store,
        &ctx.prepared.context,
        ctx.workflow.run_learn,
        &ctx.prepared.artifacts,
    )?;
    let learn_ref = learn_stored
        .as_ref()
        .map(|(p, l)| (p.as_str(), l.as_path()));
    let timing = ctx.client.attach_run_timing_for_session();
    let acp_result = ctx
        .client
        .run_kpop_multiturn(
            &ctx.prepared.artifacts.work_dir,
            &ctx.prepared.artifacts.log_path("kpop"),
            learn_ref,
            LEARN_MIN_ELAPSED_MS,
            ctx.state,
            &ctx.prepared.grounding_backup,
        )
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
    let store = kpop_prompt_store(&kpop, workflow)?;
    let emit_stdout_markdown = shared.acp_stdout_markdown_enabled();
    let mut client = build_agent(shared, workflow, emit_stdout_markdown);
    client.ensure_authenticated().map_err(|e| e.to_string())?;

    let prepared = prepare_kpop_run(&kpop)?;
    repo_checks::run_repo_workspace_gates(&prepared.artifacts.work_dir, RepoGateOutput::Tagged)?;

    kpop_emit_startup(&kpop, shared, &prepared.artifacts)?;

    let builder = KpopTurnPrompts {
        store: &store,
        base: &prepared.context,
        request_text: &prepared.text,
    };
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

    merge_acp_with_grounding_restore(
        acp_result,
        &prepared.artifacts.work_dir,
        &prepared.grounding_backup,
    )?;

    print_stdout_line(MALVIN_WHO, "DONE");
    Ok(())
}

pub fn kpop_emit_startup(
    kpop: &KpopArgs,
    shared: &SharedOpts,
    artifacts: &RunArtifacts,
) -> Result<(), String> {
    emit_run_startup_sequence(artifacts, shared.tee_startup_stdout(), &kpop.request)
}

pub fn kpop_learn_bundle(
    store: &PromptStore,
    context: &HashMap<String, String>,
    run_learn: bool,
    artifacts: &RunArtifacts,
) -> Result<Option<(String, PathBuf)>, String> {
    if !run_learn {
        return Ok(None);
    }
    let learn_prompt = store
        .render("learn.md", context)
        .map_err(|e: PromptError| e.0)?;
    let learn_log = artifacts.log_path("learn_kpop");
    Ok(Some((learn_prompt, learn_log)))
}

#[test]
fn stringify_kpop_flow_helpers() {
    let _ = stringify!(crate::cli::timing_merge::merge_acp_with_grounding_restore);
    let _ = stringify!(crate::cli::kpop_flow::kpop_prompt_store);
    let _ = stringify!(crate::cli::kpop_flow::prepare_kpop_run);
    let _ = stringify!(crate::artifacts::RunArtifacts::exp_log_path);
    let _ = stringify!(crate::cli::kpop_flow::KpopAcpMultiturnCtx);
    let _ = stringify!(crate::cli::kpop_flow::kpop_emit_startup);
    let _ = stringify!(crate::cli::kpop_flow::kpop_learn_bundle);
    let _ = stringify!(crate::cli::kpop_flow::KpopTurnPrompts);
}

#[test]
fn legacy_timing_error_order_masks_acp_when_both_fail() {
    let acp: Result<(), String> = Err("acp".into());
    let timing: std::io::Result<()> = Err(std::io::Error::other("timing"));
    let legacy = (|| {
        timing.map_err(|e| e.to_string())?;
        acp
    })();
    assert!(
        legacy.unwrap_err().contains("timing"),
        "legacy order should surface timing error, masking ACP (H1)"
    );
}

#[test]
fn merge_acp_prefers_acp_error_when_both_fail() {
    let timing: std::io::Result<()> = Err(std::io::Error::other("timing"));
    let merged = super::timing_merge::merge_acp_and_timing_results(Err("acp".into()), timing);
    assert_eq!(merged, Err("acp".into()));
}

#[test]
fn kpop_turn_prompts_include_kpop_common_and_exp_log() {
    let tmp = tempfile::tempdir().unwrap();
    let store = PromptStore::with_root(tmp.path().to_path_buf());
    store.ensure_defaults().unwrap();
    let mut base = HashMap::new();
    for (k, v) in [
        ("plan_path", "./_malvin/run42/plan.md"),
        ("grounding_path", "./grounding.md"),
        ("kpop_log_dir", "./_malvin/run42/_kpop"),
        ("review_path", "./review.md"),
        ("result_path", "./_malvin/run42/result.md"),
        ("exp_log", "_malvin/run42/_kpop/exp_log_run42.md"),
    ] {
        base.insert(k.to_string(), v.to_string());
    }
    let mut turn = KpopTurnPrompts {
        store: &store,
        base: &base,
        request_text: "do the thing",
    };
    let kpop = turn.kpop_block(2, 10).unwrap();
    let kpop_header = kpop.find("AFTER EVERY REQUEST").expect("header marker");
    let kpop_common = kpop
        .find("### KPop: Apply this method to the user's problem.")
        .expect("common marker");
    let kpop_body = kpop.find("# This KPOP turn").expect("body marker");
    assert!(
        kpop_header < kpop_common && kpop_common < kpop_body,
        "kpop prompt section order must be header, common, body"
    );
    assert!(kpop.contains("Restate the problem clearly"));
    assert!(kpop.contains("Hypothesize"));
    assert!(kpop.contains("_malvin/run42/_kpop/exp_log_run42.md"));
    let mbc2 = turn.mbc2_pure().unwrap();
    let mbc2_header = mbc2.find("AFTER EVERY REQUEST").expect("header marker");
    let mbc2_common = mbc2
        .find("### KPop: Apply this method to the user's problem.")
        .expect("common marker");
    let mbc2_body = mbc2.find("# Pure MBC2 turn").expect("body marker");
    assert!(
        mbc2_header < mbc2_common && mbc2_common < mbc2_body,
        "mbc2 prompt section order must be header, common, body"
    );
    assert!(mbc2.contains("Restate the problem clearly"));
    assert!(mbc2.contains("_malvin/run42/_kpop/exp_log_run42.md"));
}
