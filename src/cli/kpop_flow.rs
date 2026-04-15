//! KPOP subcommand: artifacts, prompt assembly, and ACP dispatch.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rand::SeedableRng;
use rand::rngs::StdRng;

use super::KpopArgs;
use super::WorkflowCliOptions;
use super::build_agent;
use super::emit_run_startup_sequence;
use super::prepare_kpop_prompt_store;
use super::repo_checks;
use super::LEARN_MIN_ELAPSED_MS;
use super::timing_merge::{emit_run_timing_after_acp, prefer_primary_string_errors};
use malvin::acp::{AgentClient, KpopFlowOnceArgs};
use malvin::artifacts::{
    RunArtifacts, backup_workspace_grounding_if_present, create_kpop_run_artifacts,
    resolve_user_request, restore_workspace_grounding,
};
use malvin::kpop_schedule::{
    KpopScheduleStep, build_scheduled_kpop_prompt, generate_kpop_schedule, schedule_requires_mbc2,
};
use malvin::orchestrator::workflow_context_paths_only;
use malvin::output::{MALVIN_WHO, print_stdout_line};
use malvin::prompts::{
    PromptError, PromptStore, merged_coding_rules, render_mbc2_for_scheduled_kpop_block,
};

fn merge_kpop_acp_with_grounding_restore(
    primary: Result<(), String>,
    work_dir: &Path,
    grounding_backup: Option<&PathBuf>,
) -> Result<(), String> {
    let restore_res = grounding_backup
        .map_or(Ok(()), |b| restore_workspace_grounding(work_dir, b));
    prefer_primary_string_errors(primary, restore_res)
}

fn kpop_schedule_and_store(
    kpop: &KpopArgs,
    workflow: WorkflowCliOptions,
) -> Result<(PromptStore, Vec<KpopScheduleStep>, bool), String> {
    let mut rng = StdRng::from_entropy();
    let schedule = generate_kpop_schedule(kpop.max_loops, kpop.p_creative, &mut rng);
    let needs_mbc2 = schedule_requires_mbc2(&schedule);
    let store = prepare_kpop_prompt_store(workflow, needs_mbc2)?;
    Ok((store, schedule, needs_mbc2))
}

pub async fn run_kpop(kpop: KpopArgs, workflow: WorkflowCliOptions) -> Result<(), String> {
    let (store, schedule, needs_mbc2) = kpop_schedule_and_store(&kpop, workflow)?;
    let mut client = build_agent(&kpop.shared, workflow);
    client.ensure_authenticated().map_err(|e| e.to_string())?;

    let (text, work_dir) = resolve_user_request(&kpop.request)?;
    let artifacts =
        create_kpop_run_artifacts(&text, Some(work_dir.as_path())).map_err(|e| e.to_string())?;

    repo_checks::run_repo_workspace_gates(&artifacts.work_dir)?;

    let grounding_backup = backup_workspace_grounding_if_present(&artifacts.work_dir)?;

    kpop_emit_startup(&kpop, &artifacts)?;

    let acp_res = kpop_run_prompt_and_finalize_timing(KpopAfterStartup {
        client: &mut client,
        workflow,
        artifacts: &artifacts,
        store: &store,
        text: &text,
        schedule: &schedule,
        needs_mbc2,
    })
    .await;
    merge_kpop_acp_with_grounding_restore(acp_res, &artifacts.work_dir, grounding_backup.as_ref())?;

    print_stdout_line(MALVIN_WHO, "DONE");
    Ok(())
}

struct KpopAfterStartup<'a> {
    client: &'a mut AgentClient,
    workflow: WorkflowCliOptions,
    artifacts: &'a RunArtifacts,
    store: &'a PromptStore,
    text: &'a str,
    schedule: &'a [KpopScheduleStep],
    needs_mbc2: bool,
}

async fn kpop_run_prompt_and_finalize_timing(ctx: KpopAfterStartup<'_>) -> Result<(), String> {
    let mut context = workflow_context_paths_only(ctx.artifacts);
    let kpop_core = ctx
        .store
        .render_prompt_only("kpop.md", &context)
        .map_err(|e: PromptError| e.0)?;
    context.insert("kpop".to_string(), kpop_core.clone());
    let rules = merged_coding_rules(ctx.store, &context);
    let kpop_body = format!("{}\n\n{}", rules.trim_end(), kpop_core.trim_end());
    let mbc2_body = if ctx.needs_mbc2 {
        render_mbc2_for_scheduled_kpop_block(ctx.store, &context).map_err(|e: PromptError| e.0)?
    } else {
        String::new()
    };
    let combined = build_scheduled_kpop_prompt(&kpop_body, &mbc2_body, ctx.text, ctx.schedule);
    let kpop_log = ctx.artifacts.log_path("kpop");
    let input = KpopAcpInput {
        artifacts: ctx.artifacts,
        combined: &combined,
        kpop_log: &kpop_log,
        store: ctx.store,
        context: &context,
        run_learn: ctx.workflow.run_learn,
        learn_min_elapsed_ms: LEARN_MIN_ELAPSED_MS,
    };

    // Match `Orchestrator::run`: run-timing stdout summary + JSON after the ACP body (grounding.md).
    let timing = ctx.client.attach_run_timing_for_session();
    let acp_result = kpop_run_acp(ctx.client, input).await;
    emit_run_timing_after_acp(ctx.client, &ctx.artifacts.run_dir, &timing, acp_result)
}

pub struct KpopAcpInput<'a> {
    artifacts: &'a RunArtifacts,
    combined: &'a str,
    kpop_log: &'a Path,
    store: &'a PromptStore,
    context: &'a HashMap<String, String>,
    run_learn: bool,
    learn_min_elapsed_ms: u64,
}

pub async fn kpop_run_acp(client: &mut AgentClient, input: KpopAcpInput<'_>) -> Result<(), String> {
    let learn_stored =
        kpop_learn_bundle(input.store, input.context, input.run_learn, input.artifacts)?;
    let learn_ref = learn_stored
        .as_ref()
        .map(|(p, l)| (p.as_str(), l.as_path()));
    let flow = KpopFlowOnceArgs {
        cwd: &input.artifacts.work_dir,
        kpop_prompt: input.combined,
        kpop_log: input.kpop_log,
        learn: learn_ref,
        learn_min_elapsed_ms: input.learn_min_elapsed_ms,
    };
    client.run_kpop_flow(&flow).await.map_err(|e| e.0)
}

pub fn kpop_emit_startup(kpop: &KpopArgs, artifacts: &RunArtifacts) -> Result<(), String> {
    emit_run_startup_sequence(artifacts, kpop.shared.tee_startup_stdout(), &kpop.request)
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
    let _ = stringify!(crate::cli::kpop_flow::merge_kpop_acp_with_grounding_restore);
    let _ = stringify!(crate::cli::kpop_flow::kpop_schedule_and_store);
    let _ = stringify!(crate::cli::kpop_flow::KpopAfterStartup);
    let _ = stringify!(crate::cli::kpop_flow::kpop_run_prompt_and_finalize_timing);
    let _ = stringify!(crate::cli::kpop_flow::kpop_emit_startup);
    let _ = stringify!(crate::cli::kpop_flow::kpop_learn_bundle);
    let _ = stringify!(crate::cli::kpop_flow::kpop_run_acp);
    let _ = stringify!(crate::cli::kpop_flow::KpopAcpInput);
}

#[test]
fn scheduled_prompt_includes_definitions_and_schedule() {
    let schedule = [KpopScheduleStep::KpopOnce];
    let s = build_scheduled_kpop_prompt("  kpop\n", "", "  user ask  ", &schedule);
    assert!(s.contains("kpop"));
    assert!(s.contains("user ask"));
    assert!(s.contains("Planned schedule:"));
    assert!(s.contains("Execution rules:"));
}

#[test]
fn hypothesis_legacy_timing_after_hint_masks_acp_when_both_fail() {
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
