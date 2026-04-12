//! KPOP subcommand: artifacts, prompt assembly, and ACP dispatch.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::KpopArgs;
use super::WorkflowCliOptions;
use super::build_agent;
use super::echo_primary_to_stdout;
use super::emit_command_line;
use super::prepare_kpop_prompt_store;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use malvin::acp::{AgentClient, KpopFlowOnceArgs};
use malvin::artifacts::{RunArtifacts, create_kpop_run_artifacts, resolve_user_request};
use malvin::log_paths::format_logs_dir;
use malvin::orchestrator::workflow_context;
use malvin::post_run_hint::finish_post_run_hint_then_return;
use malvin::prompts::{PromptError, PromptStore};
use malvin::run_timing::{self, RunTiming};

pub async fn run_kpop(kpop: KpopArgs, workflow: WorkflowCliOptions) -> Result<(), String> {
    let store = prepare_kpop_prompt_store(workflow, kpop.p_creative)?;
    let mut client = build_agent(&kpop.shared, workflow);
    client.ensure_authenticated().map_err(|e| e.to_string())?;

    let (text, work_dir) = resolve_user_request(&kpop.request)?;
    let artifacts =
        create_kpop_run_artifacts(&text, Some(work_dir.as_path())).map_err(|e| e.to_string())?;

    kpop_emit_startup(&kpop, &artifacts)?;

    kpop_run_prompt_and_post_run_hint(KpopAfterStartup {
        client: &mut client,
        kpop: &kpop,
        workflow,
        artifacts: &artifacts,
        store: &store,
        text: &text,
    })
    .await?;

    println!("DONE");
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

fn attach_kpop_run_timing(client: &mut AgentClient) -> Arc<Mutex<RunTiming>> {
    let timing = RunTiming::new_arc();
    client.set_run_timing(Some(Arc::clone(&timing)));
    timing
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .mark_wall_start(Instant::now());
    timing
}

async fn kpop_run_prompt_and_post_run_hint(ctx: KpopAfterStartup<'_>) -> Result<(), String> {
    let context = workflow_context(ctx.artifacts);
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

    // Match `Orchestrator::run`: run-timing stderr + JSON, then post-run hint (grounding.md).
    let timing = attach_kpop_run_timing(ctx.client);
    let acp_result = kpop_run_acp(ctx.client, input).await;
    let _ = run_timing::finalize_and_emit_run_timing(&ctx.artifacts.run_dir, &timing);
    ctx.client.set_run_timing(None);
    finish_post_run_hint_then_return(&ctx.artifacts.run_dir, acp_result)
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

pub async fn kpop_run_acp(
    client: &mut AgentClient,
    input: KpopAcpInput<'_>,
) -> Result<(), String> {
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
    echo_primary_to_stdout(&artifacts.plan_path, kpop.shared.tee_startup_stdout())?;
    emit_command_line(&artifacts.run_dir, kpop.shared.tee_startup_stdout())?;
    println!("Logs: {}", format_logs_dir(&artifacts.run_dir)?);
    Ok(())
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
    let _ = stringify!(crate::cli::kpop_flow::KpopAfterStartup);
    let _ = stringify!(crate::cli::kpop_flow::kpop_run_prompt_and_post_run_hint);
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
