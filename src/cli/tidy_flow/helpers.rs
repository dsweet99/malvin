use std::collections::HashMap;
use std::path::Path;

use malvin::acp::{AgentClient, CoderPromptOptions};
use malvin::artifacts;
use malvin::artifacts::{
    GroundingBackup, RunArtifacts, backup_workspace_grounding_if_present, create_run_artifacts_from_text,
};
use malvin::orchestrator::{should_run_learn_check, workflow_context};
use malvin::prompts::{PromptError, PromptStore, HEADER_MD, merged_coding_rules};
use malvin::run_timing::TimingPhase;

use super::{
    LEARN_MIN_ELAPSED_MS, SharedOpts, WorkflowCliOptions, build_agent,
    emit_run_startup_sequence,
};
use super::timing_merge;
use super::repo_checks::{RepoGateOutput, run_repo_workspace_gates};

type TidyRunPrep = (
    AgentClient,
    RunArtifacts,
    GroundingBackup,
    PromptStore,
    HashMap<String, String>,
    bool,
);

pub struct TidyAcpInput<'a> {
    pub(crate) client: &'a mut AgentClient,
    pub(crate) artifacts: &'a RunArtifacts,
    pub(crate) store: &'a PromptStore,
    pub(crate) context: &'a HashMap<String, String>,
    pub(crate) run_learn: bool,
}

pub struct TidyPromptRestore<'a> {
    pub(crate) prompt: &'a str,
    pub(crate) label: &'a str,
    pub(crate) phase: TimingPhase,
    pub(crate) grounding_backup: &'a GroundingBackup,
    pub(crate) restore_context: &'a str,
}

pub fn prepare_tidy_prompt_store() -> Result<PromptStore, String> {
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    store
        .validate_exists(HEADER_MD)
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("tidy.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("coding_rules.md")
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

pub fn compose_tidy_prompt(
    store: &PromptStore,
    context: &HashMap<String, String>,
) -> Result<String, String> {
    let header = store
        .render_prompt_only(HEADER_MD, context)
        .map_err(|e: PromptError| e.0)?;
    let rules = merged_coding_rules(store, context).map_err(|e: PromptError| e.0)?;
    let tidy = store
        .render("tidy.md", context)
        .map_err(|e: PromptError| e.0)?;
    Ok(format!(
        "{}\n\n{}\n\n{}",
        header.trim_end(),
        rules.trim_end(),
        tidy.trim_end()
    ))
}

pub async fn run_tidy_prompt(
    input: &mut TidyAcpInput<'_>,
    prompt: &str,
    kind: &str,
    phase: TimingPhase,
) -> Result<(), String> {
    input
        .client
        .run_coder_prompt(
            prompt,
            &input.artifacts.log_path(kind),
            kind,
            CoderPromptOptions {
                llm_phase: Some(phase),
                skip_repo_style: true,
                do_trace_split: None,
                stdout_bracket_label: None,
            },
        )
        .await
        .map_err(|e| e.to_string())
}

pub async fn run_tidy_acp(
    input: &mut TidyAcpInput<'_>,
    prompt: &str,
    grounding_backup: &GroundingBackup,
) -> Result<(), String> {
    let timing = input.client.attach_run_timing_for_session();
    let begin_res = input
        .client
        .begin_coder_session(&input.artifacts.work_dir)
        .await;
    if let Err(e) = begin_res {
        input.client.set_run_timing(None);
        return Err(e.to_string());
    }

    let mut acp_result = run_tidy_and_learn(input, prompt, &timing, grounding_backup).await;
    let end_result = input
        .client
        .end_coder_session()
        .await
        .map_err(|e| e.to_string());
    if end_result.is_err() {
        if acp_result.is_ok() {
            acp_result = end_result;
        } else {
            acp_result = Err(format!("{acp_result:?} end_coder_session: {end_result:?}"));
        }
    }

    timing_merge::emit_run_timing_after_acp(
        input.client,
        &input.artifacts.run_dir,
        &timing,
        acp_result,
    )
}

pub async fn run_tidy_and_learn(
    input: &mut TidyAcpInput<'_>,
    prompt: &str,
    timing: &std::sync::Arc<std::sync::Mutex<malvin::run_timing::RunTiming>>,
    grounding_backup: &GroundingBackup,
) -> Result<(), String> {
    run_tidy_prompt_with_restore(
        input,
        TidyPromptRestore {
            prompt,
            label: "tidy",
            phase: TimingPhase::Implement,
            grounding_backup,
            restore_context: "tidy",
        },
    )
    .await?;
    if input.run_learn {
        let elapsed_ms = timing
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .elapsed_so_far()
            .as_millis();
        if should_run_learn_check(
            LEARN_MIN_ELAPSED_MS,
            u64::try_from(elapsed_ms).unwrap_or(u64::MAX),
        ) {
            let learn_prompt = input
                .store
                .render("learn.md", input.context)
                .map_err(|e: PromptError| e.0)?;
            run_tidy_prompt_with_restore(
                input,
                TidyPromptRestore {
                    prompt: &learn_prompt,
                    label: "learn",
                    phase: TimingPhase::Learn,
                    grounding_backup,
                    restore_context: "learn",
                },
            )
            .await?;
        }
    }
    Ok(())
}

pub async fn run_tidy_prompt_with_restore(
    input: &mut TidyAcpInput<'_>,
    request: TidyPromptRestore<'_>,
) -> Result<(), String> {
    let acp_result = run_tidy_prompt(input, request.prompt, request.label, request.phase).await;
    let restore_result = artifacts::restore_workspace_grounding(&input.artifacts.work_dir, request.grounding_backup)
        .map_err(|e| format!("tidy restore failed after {}: {e}", request.restore_context));
    match (acp_result, restore_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(e), Ok(())) | (Ok(()), Err(e)) => Err(e),
        (Err(e), Err(restore_error)) => Err(format!(
            "{e}; tidy restore failed after {}: {restore_error}",
            request.restore_context
        )),
    }
}

pub fn tidy_prompt_context(
    artifacts: &RunArtifacts,
) -> Result<(PromptStore, HashMap<String, String>), String> {
    let store = prepare_tidy_prompt_store()?;
    let context = workflow_context(artifacts, &store).map_err(|e: PromptError| e.0)?;
    Ok((store, context))
}

pub fn prepare_tidy_run(
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    run_learn: bool,
) -> Result<TidyRunPrep, String> {
    let client = build_agent(shared, workflow, shared.acp_stdout_markdown_enabled());
    client.ensure_authenticated().map_err(|e| e.to_string())?;

    let artifacts =
        create_run_artifacts_from_text("tidy", Some(Path::new("."))).map_err(|e| e.to_string())?;
    run_repo_workspace_gates(&artifacts.work_dir, RepoGateOutput::Tagged)?;
    emit_run_startup_sequence(&artifacts, shared.tee_startup_stdout(), "tidy")?;
    let grounding_backup = backup_workspace_grounding_if_present(&artifacts.work_dir)?;
    let (store, context) = tidy_prompt_context(&artifacts)?;

    Ok((
        client,
        artifacts,
        grounding_backup,
        store,
        context,
        run_learn,
    ))
}

pub fn merge_tidy_timing(
    result: Result<(), String>,
    artifacts: &RunArtifacts,
    grounding_backup: &GroundingBackup,
) -> Result<(), String> {
    timing_merge::merge_acp_with_grounding_restore_and_check_abort(
        result,
        &artifacts.work_dir,
        grounding_backup,
        &artifacts.artifact_result_md(),
    )?;
    Ok(())
}
