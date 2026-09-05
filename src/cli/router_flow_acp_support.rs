use crate::agent_backend::agent_backend_start_coder_session;
use crate::artifacts::{
    GitignoreBackup, MalvinChecksBackup, MalvinConfigWorkspaceBackup, RunArtifacts,
    SessionDotfileBackups, VisionBackup, ensure_gate_exp_log_file,
};
use crate::router_flow::router_flow_no_work::chat_has_malvin_done;
use crate::router_flow::router_flow_prompt;
use std::path::Path;

use super::RouterAcpIterationInput;
use super::router_flow_coder_prompts::{
    RouterInitialCoderPrompt, run_router_b_coder_prompt, run_router_initial_coder_prompt,
};

pub(crate) struct RouterTurnsOutcome {
    pub iteration_backups: SessionDotfileBackups,
    pub done: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RouterExitSummarize {
    Run,
    Skip,
}

pub(crate) fn router_iteration_log_path(
    artifacts: &RunArtifacts,
    agent_loop: usize,
) -> std::path::PathBuf {
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
    let model = input.shared.model.canonical();
    let creative = input.shared.sample_creative_this_iteration();
    let _exp_log = ensure_gate_exp_log_file(input.artifacts, 1).map_err(|e| e.to_string())?;
    let iteration_backups = deliver_router_initial_turn(input, log_path, creative).await?;
    let done = finish_router_a_maybe_b(input, log_path, &model, creative).await?;
    Ok(RouterTurnsOutcome {
        iteration_backups,
        done,
    })
}

/// Bind/send the aggregated initial prompts (header + kpop + optional mbc2 + `router_a`).
async fn deliver_router_initial_turn(
    input: &mut RouterAcpIterationInput<'_>,
    log_path: &Path,
    creative: bool,
) -> Result<SessionDotfileBackups, String> {
    let work_dir = input.artifacts.work_dir.as_path();
    let model = input.shared.model.canonical();
    let include_header = !input.client.header_delivered;
    let initial =
        router_flow_prompt::build_router_initial_prompt(router_flow_prompt::RouterInitialPromptInput {
            store: input.prompt_store,
            artifacts: input.artifacts,
            model: &model,
            git: input.shared.git,
            gates: input.shared.gates,
            no_kpop: input.shared.no_kpop,
            creative,
            max_hypotheses: input.max_hypotheses,
            include_header,
        })?;

    if include_header {
        input.client.bind_session_header_parts(
            initial.body,
            log_path.to_path_buf(),
            &initial.stdout_label,
            initial.log_who,
        );
        let begin = agent_backend_start_coder_session(input.client, work_dir);
        let snapshot = async {
            SessionDotfileBackups::snapshot_after_ensuring_home_config(work_dir)
                .map_err(|e| e.to_string())
        };
        let (begin_res, snapshot_res) = tokio::join!(begin, snapshot);
        begin_res.map_err(|e| e.to_string())?;
        snapshot_res
    } else {
        let iteration_backups =
            SessionDotfileBackups::snapshot_after_ensuring_home_config(work_dir)?;
        super::begin_coder_session_if_needed(input.client, work_dir).await?;
        run_router_initial_coder_prompt(RouterInitialCoderPrompt {
            client: input.client,
            prompt: &initial.body,
            log_path,
            stdout_bracket_label: &initial.stdout_label,
            log_who: initial.log_who,
        })
        .await?;
        Ok(iteration_backups)
    }
}

async fn finish_router_a_maybe_b(
    input: &mut RouterAcpIterationInput<'_>,
    log_path: &Path,
    model: &str,
    creative: bool,
) -> Result<bool, String> {
    let chat = input
        .client
        .last_coder_prompt_agent_response()
        .unwrap_or_default();
    let done = chat_has_malvin_done(&chat);
    if !done {
        let no_kpop = input.shared.no_kpop;
        let router_b =
            router_flow_prompt::build_router_b_prompt(router_flow_prompt::RouterBPromptInput {
                store: input.prompt_store,
                artifacts: input.artifacts,
                model,
                git: input.shared.git,
                creative,
                no_kpop,
            })?;
        run_router_b_coder_prompt(
            input.client,
            &router_b,
            log_path,
            router_flow_prompt::router_b_prompt_label(crate::prompts::RouterBPromptFlags {
                creative,
                no_kpop,
            }),
        )
        .await?;
    }
    Ok(done)
}

#[cfg(test)]
#[path = "router_flow_acp_support_tests.rs"]
mod router_flow_acp_support_tests;

#[cfg(test)]
#[path = "router_flow_acp_support_kiss_cov_tests.rs"]
mod router_flow_acp_support_kiss_cov_tests;
