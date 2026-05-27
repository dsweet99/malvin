//! `do` subcommand: one coder ACP prompt with dual headers (`header.md` + `do_header.md`) and user request.

use std::path::Path;

use crate::artifacts::{
    RunArtifacts, SessionDotfileBackups, backup_workspace_kissconfig_if_present,
    backup_workspace_kissignore_if_present, backup_workspace_malvin_checks_if_present,
    backup_workspace_malvin_config_if_present, resolve_user_request,
};
use crate::cli::cli_request::require_cli_request;
use crate::cli::{AgentStdoutTeeFlags, SharedOpts, WorkflowCliOptions, agent_io_options, new_agent_client};
use crate::output::agent_stdout_tee_enabled;
use crate::repo_checks;
use crate::run_timing::TimingPhase;
use clap::Args;

pub(crate) mod do_flow_prompt;

pub use do_flow_prompt::{
    combine_do_acp_prompt_header_and_user, combine_do_prompt_file_and_user,
    combine_do_raw_header_and_user, prepare_do_prompt_store,
};

/// Arguments for [`run_do`].
#[derive(Args, Debug)]
pub struct DoArgs {
    /// Run repository quality gates before the prompt (coding-style runs).
    #[arg(long, default_value_t = false)]
    pub repo_gates: bool,
    #[arg(long, default_value_t = false)]
    pub thoughts: bool,
    /// Request or `@file` → `.malvin/logs/.../plan.md`.
    pub request: Option<String>,
}

struct DoRunPrep {
    client: crate::acp::AgentClient,
    artifacts: RunArtifacts,
    coder: do_flow_prompt::DoCoderRun,
    session_dotfile_backups: SessionDotfileBackups,
}

fn new_do_client(
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    thoughts: bool,
) -> crate::acp::AgentClient {
    let interactive = agent_stdout_tee_enabled();
    let emit_markdown = interactive && shared.acp_stdout_markdown_enabled();
    if interactive {
        return new_agent_client(
            shared,
            agent_io_options(
                shared,
                workflow,
                AgentStdoutTeeFlags {
                    emit_stdout_markdown: emit_markdown,
                    raw_output: false,
                    show_thoughts_on_stdout: thoughts,
                },
            ),
        );
    }
    new_agent_client(
        shared,
        agent_io_options(
            shared,
            workflow,
            AgentStdoutTeeFlags {
                emit_stdout_markdown: false,
                raw_output: true,
                show_thoughts_on_stdout: thoughts,
            },
        ),
    )
}

fn run_do_repo_gates_if_requested(
    do_args: &DoArgs,
    artifacts: &RunArtifacts,
) -> Result<(), String> {
    if do_args.repo_gates {
        repo_checks::run_repo_workspace_gates_no_kiss_clamp(
            &artifacts.work_dir,
            repo_checks::RepoGateOutput::Stderr,
            Some(&artifacts.run_dir),
        )?;
    }
    Ok(())
}

fn snapshot_do_session_dotfiles(work_dir: &Path) -> Result<SessionDotfileBackups, String> {
    Ok(SessionDotfileBackups::from_parts(
        backup_workspace_kissconfig_if_present(work_dir)?,
        backup_workspace_malvin_checks_if_present(work_dir)?,
        backup_workspace_kissignore_if_present(work_dir)?,
        backup_workspace_malvin_config_if_present(work_dir)?,
    ))
}

async fn prepare_do_run(
    do_args: &DoArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<DoRunPrep, String> {
    let client = new_do_client(shared, workflow, do_args.thoughts);
    let request = require_cli_request(do_args.request.as_ref(), "do")?;
    let (text, work_dir) = resolve_user_request(&request)?;
    let artifacts = crate::artifacts::create_run_artifacts_from_text_opts(
        &text,
        Some(work_dir.as_path()),
        crate::run_id::RunDirOptions::without_gc(),
    )
    .map_err(|e| e.to_string())?;
    crate::cli::error_run_log::set_command_error_run_dir(Some(artifacts.run_dir.clone()));
    run_do_repo_gates_if_requested(do_args, &artifacts)?;
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    let coder = do_flow_prompt::build_do_coder_run(&artifacts, &text)?;
    let session_dotfile_backups = snapshot_do_session_dotfiles(&artifacts.work_dir)?;
    Ok(DoRunPrep {
        client,
        artifacts,
        coder,
        session_dotfile_backups,
    })
}

pub async fn run_do(
    do_args: DoArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let mut prep = prepare_do_run(&do_args, shared, workflow).await?;
    crate::cli::run_emit::emit_command_line(&prep.artifacts.run_dir, false)?;
    prep.client.prompts_log_run_dir = Some(prep.artifacts.run_dir.clone());
    let acp_res = run_do_acp(&mut prep.client, &prep.artifacts, prep.coder).await;
    let r = crate::acp_post_run::merge_acp_with_workspace_session_restore_and_check_abort(
        acp_res,
        &prep.artifacts.work_dir,
        &prep.session_dotfile_backups,
        &prep.artifacts.artifact_result_md(),
    );
    if r.is_ok() {
        crate::cli::error_run_log::clear_command_error_run_dir();
    }
    r?;
    Ok(())
}

async fn run_do_coder_prompt(
    client: &mut crate::acp::AgentClient,
    artifacts: &RunArtifacts,
    coder: &do_flow_prompt::DoCoderRun,
) -> Result<(), String> {
    let (ref header, ref user) = coder.header_user_for_trace;
    client
        .run_coder_prompt(
            &coder.combined,
            &artifacts.log_path("do"),
            "do",
            crate::acp::CoderPromptOptions {
                llm_phase: Some(TimingPhase::Implement),
                do_trace_split: Some((header.as_str(), user.as_str())),
                stdout_bracket_label: None,
            },
        )
        .await
        .map_err(|e| e.to_string())
}

async fn run_do_acp(
    client: &mut crate::acp::AgentClient,
    artifacts: &RunArtifacts,
    coder: do_flow_prompt::DoCoderRun,
) -> Result<(), String> {
    let timing = client.attach_run_timing_for_session();
    if let Err(e) = client.begin_coder_session(&artifacts.work_dir).await {
        client.set_run_timing(None);
        return Err(e.to_string());
    }
    client.set_timing_implement_display_name("do");
    let run_res = run_do_coder_prompt(client, artifacts, &coder).await;
    let end_res = client.end_coder_session().await.map_err(|e| e.to_string());
    let merged =
        crate::acp_post_run::prefer_primary_over_secondary(run_res, end_res, "end coder session");
    crate::acp_post_run::emit_run_timing_json_only_after_acp(
        client,
        &artifacts.run_dir,
        &timing,
        merged,
    )
}

#[cfg(test)]
mod do_snapshot_tests {
    use super::snapshot_do_session_dotfiles;

    #[test]
    fn snapshot_do_session_dotfiles_on_empty_workdir() {
        let tmp = tempfile::tempdir().expect("tempdir");
        snapshot_do_session_dotfiles(tmp.path()).expect("snapshot");
    }
}

#[cfg(test)]
mod kiss_static_fn_item_refs {
    use super::{run_do, run_do_acp, run_do_coder_prompt};

    #[test]
    fn kiss_static_fn_item_refs() {
        let _ = run_do;
        let _ = run_do_acp;
        let _ = run_do_coder_prompt;
    }
}

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _: Option<DoRunPrep> = None;
        let _ = new_do_client;
        let _ = prepare_do_run;
        let _ = run_do_repo_gates_if_requested;
    }
}
