use crate::artifacts::{
    GitignoreBackup, MalvinChecksBackup, MalvinConfigWorkspaceBackup, RunArtifacts,
    SessionDotfileBackups, VisionBackup, ensure_gate_exp_log_file,
};
use crate::router_flow::router_flow_no_work::chat_has_malvin_done;
use crate::router_flow::router_flow_prompt;
use std::path::Path;

use super::RouterAcpIterationInput;
use super::router_flow_coder_prompts::{
    run_router_a_coder_prompt, run_router_b_coder_prompt, run_router_kpop_common_coder_prompt,
    run_router_mbc2_coder_prompt,
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
    let work_dir = input.artifacts.work_dir.as_path();
    let iteration_backups = SessionDotfileBackups::snapshot_after_ensuring_home_config(work_dir)?;
    let model = input.shared.model.canonical();
    let creative = input.shared.sample_creative_this_iteration();
    let _exp_log = ensure_gate_exp_log_file(input.artifacts, 1).map_err(|e| e.to_string())?;
    let no_kpop = input.shared.no_kpop;
    let kpop = router_flow_prompt::build_router_kpop_common_prompt(
        router_flow_prompt::RouterKpopCommonPromptInput {
            store: input.prompt_store,
            artifacts: input.artifacts,
            model: &model,
            git: input.shared.git,
            max_hypotheses: input.max_hypotheses,
            no_kpop,
        },
    )?;
    if !kpop.is_empty() {
        run_router_kpop_common_coder_prompt(
            input.client,
            &kpop,
            log_path,
            router_flow_prompt::kpop_common_prompt_label(no_kpop),
        )
        .await?;
    }
    if creative {
        let mbc2 =
            router_flow_prompt::build_router_mbc2_prompt(input.prompt_store, input.artifacts)?;
        run_router_mbc2_coder_prompt(input.client, &mbc2, log_path).await?;
    }
    run_router_a_coder_prompt(
        input.client,
        &router_flow_prompt::build_router_a_prompt(router_flow_prompt::RouterAPromptInput {
            store: input.prompt_store,
            artifacts: input.artifacts,
            model: &model,
            git: input.shared.git,
            gates: input.shared.gates,
            no_kpop,
        })?,
        log_path,
        router_flow_prompt::router_a_prompt_label(no_kpop),
    )
    .await?;
    let done = finish_router_a_maybe_b(input, log_path, &model, creative).await?;
    Ok(RouterTurnsOutcome {
        iteration_backups,
        done,
    })
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
