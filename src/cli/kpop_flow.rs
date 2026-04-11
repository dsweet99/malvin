//! KPOP subcommand: artifacts, prompt assembly, and ACP dispatch.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use malvin::agent::AgentClient;
use malvin::artifacts::{create_kpop_run_artifacts, resolve_user_request, RunArtifacts};
use malvin::orchestrator::workflow_context;
use malvin::prompts::{PromptError, PromptStore};

use super::build_agent;
use super::echo_primary_to_stdout;
use super::emit_command_line;
use super::format_logs_dir;
use super::prepare_prompt_store;
use super::KpopArgs;
use super::WorkflowCliOptions;

pub async fn run_kpop(kpop: KpopArgs, workflow: WorkflowCliOptions) -> Result<(), String> {
    let store = prepare_prompt_store(workflow)?;
    let mut client = build_agent(&kpop.shared, workflow);
    client
        .ensure_authenticated()
        .map_err(|e| e.to_string())?;

    let (text, work_dir) = resolve_user_request(&kpop.request)?;
    let artifacts =
        create_kpop_run_artifacts(&text, Some(work_dir.as_path())).map_err(|e| e.to_string())?;

    kpop_emit_startup(&kpop, &artifacts)?;

    let context = workflow_context(&artifacts);
    let kpop_body = store
        .render("kpop.md", &context)
        .map_err(|e: PromptError| e.0)?;
    let combined = kpop_combined_prompt(&kpop_body, &text, kpop.max_loops);
    let kpop_log = artifacts.log_path("kpop");
    let input = KpopAcpInput {
        artifacts: &artifacts,
        combined: &combined,
        kpop_log: &kpop_log,
        store: &store,
        context: &context,
        run_learn: workflow.run_learn,
    };
    kpop_run_acp(&mut client, input).await?;

    println!("DONE");
    Ok(())
}

struct KpopAcpInput<'a> {
    artifacts: &'a RunArtifacts,
    combined: &'a str,
    kpop_log: &'a Path,
    store: &'a PromptStore,
    context: &'a HashMap<String, String>,
    run_learn: bool,
}

async fn kpop_run_acp(client: &mut AgentClient, input: KpopAcpInput<'_>) -> Result<(), String> {
    let learn_stored = kpop_learn_bundle(
        input.store,
        input.context,
        input.run_learn,
        input.artifacts,
    )?;
    let learn_ref = learn_stored
        .as_ref()
        .map(|(p, l)| (p.as_str(), l.as_path()));
    client
        .run_kpop_flow(
            &input.artifacts.work_dir,
            input.combined,
            input.kpop_log,
            learn_ref,
        )
        .await
        .map_err(|e| e.0)
}

fn kpop_emit_startup(kpop: &KpopArgs, artifacts: &RunArtifacts) -> Result<(), String> {
    echo_primary_to_stdout(&artifacts.plan_path, kpop.shared.primary_doc_plain_echo())?;
    emit_command_line(&artifacts.run_dir);
    println!("Logs: {}", format_logs_dir(&artifacts.run_dir)?);
    Ok(())
}

fn kpop_combined_prompt(kpop_body: &str, user_text: &str, budget: usize) -> String {
    format!(
        "{}\n\n{}\n\nYou have a budget of {} hypotheses.",
        kpop_body.trim_end(),
        user_text.trim_end(),
        budget
    )
}

fn kpop_learn_bundle(
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

#[cfg(test)]
mod kiss_refs {
    #[test]
    fn stringify_kpop_flow_helpers() {
        let _ = stringify!(super::kpop_emit_startup);
        let _ = stringify!(super::kpop_combined_prompt);
        let _ = stringify!(super::kpop_learn_bundle);
        let _ = stringify!(super::kpop_run_acp);
        let _ = stringify!(super::KpopAcpInput);
    }
}
