use std::path::Path;

use malvin::acp::{AgentClient, CoderPromptOptions};
use malvin::artifacts::{
    RunArtifacts, backup_workspace_grounding_if_present, create_run_artifacts_from_text,
};
use malvin::orchestrator::workflow_context_paths_only;
use malvin::output::{MALVIN_WHO, print_stdout_line};
use malvin::prompts::{PromptError, PromptStore};
use malvin::run_timing::TimingPhase;

use super::{SharedOpts, WorkflowCliOptions, run_emit, timing_merge};

const GROUND_REQUEST: &str = "ground";

struct GroundSession {
    artifacts: RunArtifacts,
    grounding_backup: malvin::artifacts::GroundingBackup,
    prompt: String,
}

fn prepare_ground_prompt_store() -> Result<PromptStore, String> {
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("write_grounding.md")
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

fn build_grounding_prompt(store: &PromptStore, artifacts: &RunArtifacts) -> Result<String, String> {
    let context = workflow_context_paths_only(artifacts);
    store
        .render("write_grounding.md", &context)
        .map_err(|e: PromptError| e.0)
}

fn prepare_ground_session() -> Result<GroundSession, String> {
    let store = prepare_ground_prompt_store()?;
    let artifacts = create_run_artifacts_from_text(GROUND_REQUEST, Some(Path::new(".")))
        .map_err(|e| e.to_string())?;
    if artifacts.work_dir.join("grounding.md").exists() {
        return Err("ABORT: grounding.md already exists".to_string());
    }
    let grounding_backup = backup_workspace_grounding_if_present(&artifacts.work_dir)?;
    let prompt = build_grounding_prompt(&store, &artifacts)?;
    Ok(GroundSession {
        artifacts,
        grounding_backup,
        prompt,
    })
}

async fn run_ground_acp(
    client: &mut AgentClient,
    artifacts: &RunArtifacts,
    prompt: &str,
) -> Result<(), String> {
    let timing = client.attach_run_timing_for_session();
    let begin_res = client
        .begin_coder_session(&artifacts.work_dir)
        .await;
    if let Err(e) = begin_res {
        client.set_run_timing(None);
        return Err(e.to_string());
    }
    timing
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .set_implement_display_name("ground");
    let acp_result = client
        .run_coder_prompt(
            prompt,
            &artifacts.log_path("ground"),
            "ground",
            CoderPromptOptions {
                llm_phase: Some(TimingPhase::Implement),
                skip_repo_style: true,
                do_trace_split: None,
                stdout_bracket_label: None,
            },
        )
        .await
        .map_err(|e| e.to_string());
    let end_result = client.end_coder_session().await.map_err(|e| e.to_string());
    let result = if let Err(e) = acp_result {
        Err(e)
    } else if let Err(e) = end_result {
        Err(e)
    } else {
        Ok(())
    };
    timing_merge::emit_run_timing_after_acp(client, &artifacts.run_dir, &timing, result)
}

pub async fn run_ground(shared: &SharedOpts, workflow: WorkflowCliOptions) -> Result<(), String> {
    let session = prepare_ground_session()?;
    crate::cli::kiss_clamp::ensure_kiss_clamp_if_needed(&session.artifacts.work_dir, true)?;
    let mut client = super::build_agent(shared, workflow, shared.acp_stdout_markdown_enabled());
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    run_emit::emit_run_startup_sequence(
        &session.artifacts,
        shared.tee_startup_stdout(),
        GROUND_REQUEST,
    )?;
    let result = run_ground_acp(&mut client, &session.artifacts, &session.prompt).await;
    timing_merge::merge_acp_with_grounding_restore_and_check_abort(
        result,
        &session.artifacts.work_dir,
        &session.grounding_backup,
        &session.artifacts.artifact_result_md(),
    )?;
    print_stdout_line(MALVIN_WHO, "DONE");
    Ok(())
}

#[test]
fn stringify_ground_flow_helpers() {
    let _ = stringify!(crate::cli::ground_cmd::prepare_ground_prompt_store);
    let _ = stringify!(crate::cli::ground_cmd::build_grounding_prompt);
    let _ = stringify!(crate::cli::ground_cmd::prepare_ground_session);
    let _ = stringify!(crate::cli::ground_cmd::GroundSession);
    let _ = stringify!(crate::cli::ground_cmd::run_ground_acp);
    let _ = stringify!(crate::cli::ground_cmd::run_ground);
}
