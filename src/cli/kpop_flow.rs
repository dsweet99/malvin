//! KPOP subcommand: artifacts, prompt assembly, and ACP dispatch.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::KpopArgs;
use super::WorkflowCliOptions;
use super::build_agent;
use super::emit_run_startup_sequence;
use super::prepare_kpop_prompt_store;
use super::timing_merge::{emit_run_timing_after_acp, prefer_primary_string_errors};
use malvin::acp::{AgentClient, KpopFlowOnceArgs};
use malvin::artifacts::{
    RunArtifacts, backup_workspace_grounding_if_present, create_kpop_run_artifacts,
    resolve_user_request, restore_workspace_grounding,
};
use malvin::orchestrator::workflow_context;
use malvin::output::{MALVIN_WHO, print_stdout_line};
use malvin::prompts::{PromptError, PromptStore};

fn merge_kpop_acp_with_grounding_restore(
    primary: Result<(), String>,
    work_dir: &Path,
    grounding_backup: Option<&PathBuf>,
) -> Result<(), String> {
    let restore_res = grounding_backup
        .map_or(Ok(()), |b| restore_workspace_grounding(work_dir, b));
    prefer_primary_string_errors(primary, restore_res)
}

pub async fn run_kpop(kpop: KpopArgs, workflow: WorkflowCliOptions) -> Result<(), String> {
    let store = prepare_kpop_prompt_store(workflow, kpop.p_creative)?;
    let mut client = build_agent(&kpop.shared, workflow);
    client.ensure_authenticated().map_err(|e| e.to_string())?;

    let (text, work_dir) = resolve_user_request(&kpop.request)?;
    let artifacts =
        create_kpop_run_artifacts(&text, Some(work_dir.as_path())).map_err(|e| e.to_string())?;

    let grounding_backup = backup_workspace_grounding_if_present(&artifacts.work_dir)?;

    kpop_emit_startup(&kpop, &artifacts)?;

    let acp_res = kpop_run_prompt_and_finalize_timing(KpopAfterStartup {
        client: &mut client,
        kpop: &kpop,
        workflow,
        artifacts: &artifacts,
        store: &store,
        text: &text,
    })
    .await;
    merge_kpop_acp_with_grounding_restore(acp_res, &artifacts.work_dir, grounding_backup.as_ref())?;

    print_stdout_line(MALVIN_WHO, "DONE");
    Ok(())
}

struct KpopAfterStartup<'a> {
    client: &'a mut AgentClient,
    kpop: &'a KpopArgs,
    workflow: WorkflowCliOptions,
    artifacts: &'a RunArtifacts,
    store: &'a PromptStore,
    text: &'a str,
}

async fn kpop_run_prompt_and_finalize_timing(ctx: KpopAfterStartup<'_>) -> Result<(), String> {
    let context = workflow_context(ctx.artifacts, ctx.store);
    let kpop_body = ctx
        .store
        .render("kpop.md", &context)
        .map_err(|e: PromptError| e.0)?;
    let combined = kpop_combined_prompt(&kpop_body, ctx.text, ctx.kpop.max_loops);
    let kpop_log = ctx.artifacts.log_path("kpop");
    let input = KpopAcpInput {
        artifacts: ctx.artifacts,
        combined: &combined,
        kpop_log: &kpop_log,
        store: ctx.store,
        context: &context,
        run_learn: ctx.workflow.run_learn,
        p_creative: ctx.kpop.p_creative,
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
    p_creative: f64,
}

pub async fn kpop_run_acp(client: &mut AgentClient, input: KpopAcpInput<'_>) -> Result<(), String> {
    let learn_stored =
        kpop_learn_bundle(input.store, input.context, input.run_learn, input.artifacts)?;
    let learn_ref = learn_stored
        .as_ref()
        .map(|(p, l)| (p.as_str(), l.as_path()));
    let mbc2_body = if malvin::kpop_creative_enabled(input.p_creative) {
        input
            .store
            .render("mbc2.md", input.context)
            .map_err(|e: PromptError| e.0)?
    } else {
        String::new()
    };
    let flow = KpopFlowOnceArgs {
        cwd: &input.artifacts.work_dir,
        kpop_prompt: input.combined,
        kpop_log: input.kpop_log,
        learn: learn_ref,
        p_creative: input.p_creative,
        mbc2_body: &mbc2_body,
    };
    client.run_kpop_flow(&flow).await.map_err(|e| e.0)
}

pub fn kpop_emit_startup(kpop: &KpopArgs, artifacts: &RunArtifacts) -> Result<(), String> {
    emit_run_startup_sequence(artifacts, kpop.shared.tee_startup_stdout(), &kpop.request)
}

pub fn kpop_combined_prompt(kpop_body: &str, user_text: &str, budget: usize) -> String {
    format!(
        "{}\n\n{}\n\nYou have a budget of {} hypotheses.",
        kpop_body.trim_end(),
        user_text.trim_end(),
        budget
    )
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
    let _ = stringify!(crate::cli::kpop_flow::KpopAfterStartup);
    let _ = stringify!(crate::cli::kpop_flow::kpop_run_prompt_and_finalize_timing);
    let _ = stringify!(crate::cli::kpop_flow::kpop_emit_startup);
    let _ = stringify!(crate::cli::kpop_flow::kpop_combined_prompt);
    let _ = stringify!(crate::cli::kpop_flow::kpop_learn_bundle);
    let _ = stringify!(crate::cli::kpop_flow::kpop_run_acp);
    let _ = stringify!(crate::cli::kpop_flow::KpopAcpInput);
}

#[test]
fn trims_sections_and_includes_budget() {
    let s = kpop_combined_prompt("  kpop\n", "  user ask  ", 7);
    assert!(s.contains("kpop"));
    assert!(s.contains("user ask"));
    assert!(s.contains("budget of 7 hypotheses"));
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
