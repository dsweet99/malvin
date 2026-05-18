use std::collections::HashMap;

use super::shared_opts::SharedOpts;
use super::timing_merge;
use super::{WorkflowCliOptions, repo_checks};
use clap::Args;
use malvin::acp::{AgentClient, CoderPromptOptions};
use malvin::artifacts::{
    RunArtifacts, SessionDotfileBackups, create_run_artifacts_from_text, resolve_user_request,
};
use malvin::orchestrator::{workflow_context, workflow_context_paths_only};
use malvin::prompts::{DO_HEADER_MD, HEADER_MD, PromptError, PromptStore};
use malvin::run_timing::{RunTiming, TimingPhase};

/// Arguments for [`run_do`].
#[derive(Args, Debug)]
pub struct DoArgs {
    /// Prepend `header.md` and allow optional injected repo style
    #[arg(long, default_value_t = false)]
    pub cooked: bool,
    /// Run repository quality gates before the prompt (coding-style runs).
    #[arg(long, default_value_t = false)]
    pub repo_gates: bool,
    #[arg(long, default_value_t = false)]
    pub thoughts: bool,
    /// Request or `@file` → `_malvin/.../plan.md`.
    pub request: String,
}

struct DoCoderRun {
    combined: String,
    header_user_for_trace: (String, String),
    skip_repo_style: bool,
}

fn prepare_do_prompt_store_validating(required_template: &str) -> Result<PromptStore, String> {
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    store
        .validate_exists(required_template)
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

pub fn prepare_do_prompt_store() -> Result<PromptStore, String> {
    prepare_do_prompt_store_validating(HEADER_MD)
}

pub fn prepare_do_raw_prompt_store() -> Result<PromptStore, String> {
    prepare_do_prompt_store_validating(DO_HEADER_MD)
}

fn do_agent_client_for_do_cmd(
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    thoughts: bool,
) -> AgentClient {
    AgentClient::new(
        shared.model.clone(),
        super::agent_io_options(
            shared,
            workflow,
            super::AgentStdoutTeeFlags {
                emit_stdout_markdown: false,
                raw_output: true,
                show_thoughts_on_stdout: thoughts,
            },
        ),
    )
}

fn finalize_do_workspace_restore_merge(
    acp_res: Result<(), String>,
    artifacts: &RunArtifacts,
    session_dotfile_backups: &SessionDotfileBackups,
) -> Result<(), String> {
    let r = timing_merge::merge_acp_with_workspace_session_restore_and_check_abort(
        acp_res,
        &artifacts.work_dir,
        session_dotfile_backups,
        &artifacts.artifact_result_md(),
    );
    if r.is_ok() {
        super::error_run_log::clear_command_error_run_dir();
    }
    r
}

fn stamp_do_timing_display_name(timing: &std::sync::Arc<std::sync::Mutex<RunTiming>>) {
    timing
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .set_implement_display_name("do");
}

fn bootstrap_do_workspace_from_request(do_args: &DoArgs) -> Result<(String, RunArtifacts), String> {
    let (text, work_dir) = resolve_user_request(&do_args.request)?;
    let artifacts =
        create_run_artifacts_from_text(&text, Some(work_dir.as_path())).map_err(|e| e.to_string())?;
    super::error_run_log::set_command_error_run_dir(Some(artifacts.run_dir.clone()));

    if do_args.repo_gates {
        repo_checks::run_repo_workspace_gates_no_kiss_clamp(
            &artifacts.work_dir,
            repo_checks::RepoGateOutput::Stderr,
            Some(&artifacts.run_dir),
        )?;
    }
    Ok((text, artifacts))
}

fn compose_do_coder_bundle(
    do_args: &DoArgs,
    text: &str,
    artifacts: &RunArtifacts,
) -> Result<DoCoderRun, String> {
    let skip_repo_style = !do_args.cooked;
    let (combined, header_user) = if do_args.cooked {
        let store = prepare_do_prompt_store()?;
        let (combined, header, user) =
            combine_do_acp_prompt_header_and_user(&store, artifacts, text)?;
        (combined, (header, user))
    } else {
        let store = prepare_do_raw_prompt_store()?;
        let (combined, header, user) = combine_do_raw_header_and_user(&store, artifacts, text)?;
        (combined, (header, user))
    };
    Ok(DoCoderRun {
        combined,
        header_user_for_trace: header_user,
        skip_repo_style,
    })
}

pub async fn run_do(
    do_args: DoArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let mut client = do_agent_client_for_do_cmd(shared, workflow, do_args.thoughts);
    let (text, artifacts) = bootstrap_do_workspace_from_request(&do_args)?;
    client.ensure_authenticated().map_err(|e| e.to_string())?;

    let coder = compose_do_coder_bundle(&do_args, &text, &artifacts)?;

    let session_dotfile_backups =
        SessionDotfileBackups::snapshot(&artifacts.work_dir).map_err(|e| e.to_string())?;
    super::run_emit::emit_command_line(&artifacts.run_dir, false)?;
    client.prompts_log_run_dir = Some(artifacts.run_dir.clone());
    let acp_res = run_do_acp(&mut client, &artifacts, coder).await;
    finalize_do_workspace_restore_merge(acp_res, &artifacts, &session_dotfile_backups)?;
    Ok(())
}

fn combine_do_prompt_file_and_user(
    store: &PromptStore,
    text: &str,
    template_file: &str,
    context: &HashMap<String, String>,
) -> Result<(String, String, String), String> {
    let header_body = store
        .render_prompt_only(template_file, context)
        .map_err(|e: PromptError| e.0)?;
    let header = header_body.trim_end().to_string();
    let user = text.trim_end().to_string();
    let combined = format!("{header}\n\n{user}");
    Ok((combined, header, user))
}

/// Renders `header.md` once; returns combined prompt plus header and user strings for ACP trace splitting.
pub fn combine_do_acp_prompt_header_and_user(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    text: &str,
) -> Result<(String, String, String), String> {
    let context = workflow_context(artifacts, store, "do").map_err(|e: PromptError| e.0)?;
    combine_do_prompt_file_and_user(store, text, HEADER_MD, &context)
}

/// Renders `do_header.md` once; same return shape for default raw `malvin do`.
pub fn combine_do_raw_header_and_user(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    text: &str,
) -> Result<(String, String, String), String> {
    let context = workflow_context_paths_only(artifacts, "do");
    combine_do_prompt_file_and_user(store, text, DO_HEADER_MD, &context)
}

fn merge_two_string_results(
    primary: Result<(), String>,
    secondary: Result<(), String>,
) -> Result<(), String> {
    primary.and_then(|()| secondary)
}
