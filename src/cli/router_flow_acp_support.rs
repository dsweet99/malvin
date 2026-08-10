use crate::artifacts::{
    ensure_gate_exp_log_file, GitignoreBackup, MalvinChecksBackup, MalvinConfigWorkspaceBackup,
    RunArtifacts, SessionDotfileBackups, VisionBackup,
};
use crate::malvin_config_file::{load_malvin_config, DEFAULT_MAX_HYPOTHESES};
use crate::router_flow::router_flow_no_work::chat_has_malvin_done;
use crate::router_flow::router_flow_prompt;
use std::path::Path;

use super::router_flow_coder_prompts::{
    run_router_a_coder_prompt, run_router_b_coder_prompt, run_router_header_coder_prompt,
    run_router_kpop_common_coder_prompt,
};
use super::RouterAcpIterationInput;

pub(crate) struct RouterTurnsOutcome {
    pub iteration_backups: SessionDotfileBackups,
    pub done: bool,
}

/// Whether the outer router loop should send `router_summarize.md` before teardown.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RouterExitSummarize {
    Run,
    Skip,
}

pub(crate) fn router_iteration_log_path(artifacts: &RunArtifacts, agent_loop: usize) -> std::path::PathBuf {
    artifacts.log_path(&format!("router_{agent_loop}"))
}

pub(crate) fn empty_iteration_backups() -> SessionDotfileBackups {
    SessionDotfileBackups::from_parts(crate::session_dotfile_backup::SessionDotfileParts {
        malvin_checks: MalvinChecksBackup::Missing,
        gitignore: GitignoreBackup::Missing,
        vision: VisionBackup::Missing,
        malvin_config_workspace: MalvinConfigWorkspaceBackup::Missing,
    })
}

pub(crate) fn snapshot_iteration_backups(work_dir: &Path) -> SessionDotfileBackups {
    SessionDotfileBackups::snapshot_after_ensuring_home_config(work_dir)
        .unwrap_or_else(|_| empty_iteration_backups())
}

pub(crate) async fn run_router_turns(
    input: &mut RouterAcpIterationInput<'_>,
    log_path: &Path,
) -> Result<RouterTurnsOutcome, String> {
    let work_dir = input.artifacts.work_dir.as_path();
    let iteration_backups = SessionDotfileBackups::snapshot_after_ensuring_home_config(work_dir)?;
    let model = input.shared.model.canonical();
    run_router_header_and_kpop(input, log_path, &model).await?;
    run_router_a_coder_prompt(
        input.client,
        &router_flow_prompt::build_router_a_prompt(router_flow_prompt::RouterAPromptInput {
            store: input.prompt_store,
            artifacts: input.artifacts,
            model: &model,
            git: input.shared.git,
            gates: input.shared.gates,
        })?,
        log_path,
    )
    .await?;
    let done = finish_router_a_maybe_b(input, log_path, &model).await?;
    Ok(RouterTurnsOutcome {
        iteration_backups,
        done,
    })
}

async fn run_router_header_and_kpop(
    input: &mut RouterAcpIterationInput<'_>,
    log_path: &Path,
    model: &str,
) -> Result<(), String> {
    let header = router_flow_prompt::build_router_header_prompt(
        router_flow_prompt::RouterHeaderPromptInput {
            store: input.prompt_store,
            artifacts: input.artifacts,
            model,
            git: input.shared.git,
        },
    )?;
    run_router_header_coder_prompt(input.client, &header, log_path).await?;
    let exp_log = ensure_gate_exp_log_file(input.artifacts, 1).map_err(|e| e.to_string())?;
    let max_hypotheses = load_malvin_config(input.artifacts.work_dir.as_path())
        .default_workflow
        .max_hypotheses_or_default();
    let max_hypotheses = if max_hypotheses == 0 {
        DEFAULT_MAX_HYPOTHESES
    } else {
        max_hypotheses
    };
    let kpop_common = router_flow_prompt::build_router_kpop_common_prompt(
        router_flow_prompt::RouterKpopCommonPromptInput {
            store: input.prompt_store,
            artifacts: input.artifacts,
            model,
            git: input.shared.git,
            max_hypotheses,
            exp_log: &exp_log,
        },
    )?;
    run_router_kpop_common_coder_prompt(input.client, &kpop_common, log_path).await
}

async fn finish_router_a_maybe_b(
    input: &mut RouterAcpIterationInput<'_>,
    log_path: &Path,
    model: &str,
) -> Result<bool, String> {
    let chat = input
        .client
        .last_coder_prompt_agent_response()
        .unwrap_or_default();
    let done = chat_has_malvin_done(&chat);
    if !done {
        let router_b = router_flow_prompt::build_router_b_prompt(router_flow_prompt::RouterBPromptInput {
            store: input.prompt_store,
            artifacts: input.artifacts,
            model,
            git: input.shared.git,
        })?;
        run_router_b_coder_prompt(input.client, &router_b, log_path).await?;
    }
    Ok(done)
}

#[cfg(test)]
#[path = "router_flow_acp_support_tests.rs"]
mod router_flow_acp_support_tests;

#[cfg(test)]
#[path = "router_flow_acp_support_kiss_cov_tests.rs"]
mod router_flow_acp_support_kiss_cov_tests;
