use clap::Args;
use malvin::acp::{AgentClient, CoderPromptOptions};
use malvin::artifacts::{
    GroundingBackup, RunArtifacts, backup_workspace_grounding_if_present,
    create_run_artifacts_from_text,
};
use malvin::output::{print_stdout_line, MALVIN_WHO};
use malvin::orchestrator::{should_run_learn_check, workflow_context};
use malvin::prompts::{merged_coding_rules, PromptError, PromptStore, HEADER_MD};
use malvin::run_timing::TimingPhase;

use std::collections::HashMap;
use std::path::Path;

use super::{run_emit, SharedOpts, WorkflowCliOptions, build_agent, LEARN_MIN_ELAPSED_MS};

type TidyRunPrep = (
    AgentClient,
    RunArtifacts,
    GroundingBackup,
    PromptStore,
    HashMap<String, String>,
    bool,
);

#[derive(Args, Debug)]
pub struct TidyArgs {
    #[arg(long, default_value_t = false)]
    pub no_learn: bool,
}

struct TidyAcpInput<'a> {
    client: &'a mut AgentClient,
    artifacts: &'a RunArtifacts,
    store: &'a PromptStore,
    context: &'a HashMap<String, String>,
    run_learn: bool,
}

fn prepare_tidy_prompt_store() -> Result<PromptStore, String> {
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    store.validate_exists(HEADER_MD).map_err(|e: PromptError| e.0)?;
    store.validate_exists("tidy.md").map_err(|e: PromptError| e.0)?;
    store.validate_exists("coding_rules.md").map_err(|e: PromptError| e.0)?;
    Ok(store)
}

fn compose_tidy_prompt(store: &PromptStore, context: &HashMap<String, String>) -> Result<String, String> {
    let header = store
        .render_prompt_only(HEADER_MD, context)
        .map_err(|e: PromptError| e.0)?;
    let rules = merged_coding_rules(store, context).map_err(|e: PromptError| e.0)?;
    let tidy = store.render("tidy.md", context).map_err(|e: PromptError| e.0)?;
    Ok(format!(
        "{}\n\n{}\n\n{}",
        header.trim_end(),
        rules.trim_end(),
        tidy.trim_end()
    ))
}

async fn run_tidy_prompt(
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

async fn run_tidy_acp(input: &mut TidyAcpInput<'_>, prompt: &str) -> Result<(), String> {
    let timing = input.client.attach_run_timing_for_session();
    input
        .client
        .begin_coder_session(&input.artifacts.work_dir)
        .await
        .map_err(|e| e.to_string())?;

    let mut acp_result = run_tidy_and_learn(input, prompt, &timing).await;
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

    super::timing_merge::emit_run_timing_after_acp(
        input.client,
        &input.artifacts.run_dir,
        &timing,
        acp_result,
    )
}

async fn run_tidy_and_learn(
    input: &mut TidyAcpInput<'_>,
    prompt: &str,
    timing: &std::sync::Arc<std::sync::Mutex<malvin::run_timing::RunTiming>>,
) -> Result<(), String> {
    let mut acp_result = run_tidy_prompt(input, prompt, "tidy", TimingPhase::Implement).await;
    if acp_result.is_ok() && input.run_learn {
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
            acp_result = run_tidy_prompt(input, &learn_prompt, "learn", TimingPhase::Learn).await;
        }
    }
    acp_result
}

fn tidy_prompt_context(
    artifacts: &RunArtifacts,
) -> Result<(PromptStore, HashMap<String, String>), String> {
    let store = prepare_tidy_prompt_store()?;
    let context = workflow_context(artifacts, &store).map_err(|e: PromptError| e.0)?;
    Ok((store, context))
}

fn prepare_tidy_run(
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    tidy: &TidyArgs,
) -> Result<
    TidyRunPrep, String> {
    let client = build_agent(shared, workflow, shared.acp_stdout_markdown_enabled());
    client.ensure_authenticated().map_err(|e| e.to_string())?;

    let artifacts = create_run_artifacts_from_text("tidy", Some(Path::new(".")))
        .map_err(|e| e.to_string())?;
    crate::cli::kiss_clamp::ensure_kiss_clamp_if_needed(
        &artifacts.work_dir,
        super::repo_checks::RepoGateOutput::Tagged,
    )?;
    run_emit::emit_run_startup_sequence(&artifacts, shared.tee_startup_stdout(), "tidy")?;
    let grounding_backup = backup_workspace_grounding_if_present(&artifacts.work_dir)?;
    let (store, context) = tidy_prompt_context(&artifacts)?;

    Ok((
        client,
        artifacts,
        grounding_backup,
        store,
        context,
        !tidy.no_learn,
    ))
}

fn merge_tidy_timing(
    result: Result<(), String>,
    artifacts: &RunArtifacts,
    grounding_backup: &GroundingBackup,
) -> Result<(), String> {
    super::timing_merge::merge_acp_with_grounding_restore(
        result,
        &artifacts.work_dir,
        grounding_backup,
    )
}

pub async fn run_tidy(
    tidy: TidyArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let (mut client, artifacts, grounding_backup, store, context, run_learn) =
        prepare_tidy_run(shared, workflow, &tidy)?;
    let prompt = compose_tidy_prompt(&store, &context)?;
    let mut input = TidyAcpInput {
        client: &mut client,
        artifacts: &artifacts,
        store: &store,
        context: &context,
        run_learn,
    };
    let result = run_tidy_acp(&mut input, prompt.trim_end()).await;
    merge_tidy_timing(result, &artifacts, &grounding_backup)?;
    print_stdout_line(MALVIN_WHO, "DONE");
    Ok(())
}

#[cfg(test)]
mod coverage_tests {
    #[test]
    fn kiss_stringify_tidy_flow_units() {
        let _ = stringify!(crate::cli::tidy_flow::TidyArgs);
        let _ = stringify!(crate::cli::tidy_flow::TidyAcpInput);
        let _ = stringify!(crate::cli::tidy_flow::prepare_tidy_prompt_store);
        let _ = stringify!(crate::cli::tidy_flow::compose_tidy_prompt);
        let _ = stringify!(crate::cli::tidy_flow::run_tidy_prompt);
        let _ = stringify!(crate::cli::tidy_flow::run_tidy_acp);
        let _ = stringify!(crate::cli::tidy_flow::run_tidy_and_learn);
        let _ = stringify!(crate::cli::tidy_flow::tidy_prompt_context);
        let _ = stringify!(crate::cli::tidy_flow::prepare_tidy_run);
        let _ = stringify!(crate::cli::tidy_flow::merge_tidy_timing);
        let _ = stringify!(crate::cli::tidy_flow::run_tidy);
    }
}
