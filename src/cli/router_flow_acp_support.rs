use crate::artifacts::{
    ensure_gate_exp_log_file, GitignoreBackup, MalvinChecksBackup, MalvinConfigBackup,
    MalvinConfigWorkspaceBackup, RunArtifacts, SessionDotfileBackups, VisionBackup,
};
use crate::malvin_config_file::{load_malvin_config, DEFAULT_MAX_HYPOTHESES};
use crate::router_flow::router_flow_no_work::all_groups_no_work_remaining;
use crate::router_flow::router_flow_parse::{
    clear_review_requirements_json, load_review_requirements, ReviewRequirements,
};
use crate::router_flow::router_flow_prompt;
use std::path::Path;

use super::router_flow_coder_prompts::{
    run_router_kpop_group_coder_prompt, run_router_requirements_coder_prompt,
    run_router_work_coder_prompt,
};
use super::RouterAcpIterationInput;

pub(crate) struct RouterTurnsOutcome {
    pub iteration_backups: SessionDotfileBackups,
    pub all_no_work: bool,
}

pub(crate) fn router_iteration_log_path(artifacts: &RunArtifacts, agent_loop: usize) -> std::path::PathBuf {
    artifacts.log_path(&format!("router_{agent_loop}"))
}

pub(crate) fn empty_iteration_backups() -> SessionDotfileBackups {
    SessionDotfileBackups::from_parts(crate::session_dotfile_backup::SessionDotfileParts {
        malvin_checks: MalvinChecksBackup::Missing,
        malvin_config: MalvinConfigBackup::Missing,
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
    let requirements_path = crate::artifacts::review_requirements_json(input.artifacts);
    clear_review_requirements_json(&requirements_path);
    run_router_requirements_coder_prompt(input.client, input.coder, log_path).await?;
    let requirements = load_review_requirements(&requirements_path)?;
    run_multi_group_kpop(input, log_path, &requirements).await?;
    let chat = input
        .client
        .last_coder_prompt_agent_response()
        .unwrap_or_default();
    let all_no_work = all_groups_no_work_remaining(&chat, requirements.groups.len());
    if !all_no_work {
        let work_body =
            router_flow_prompt::build_router_work_prompt(router_flow_prompt::RouterWorkPromptInput {
                store: input.prompt_store,
                artifacts: input.artifacts,
                model: &input.shared.model,
                git: input.shared.git,
                gates: input.shared.gates,
            })?;
        run_router_work_coder_prompt(input.client, &work_body, log_path).await?;
    }
    Ok(RouterTurnsOutcome {
        iteration_backups,
        all_no_work,
    })
}

async fn run_multi_group_kpop(
    input: &mut RouterAcpIterationInput<'_>,
    log_path: &Path,
    requirements: &ReviewRequirements,
) -> Result<(), String> {
    let exp_log = ensure_gate_exp_log_file(input.artifacts, 1).map_err(|e| e.to_string())?;
    let want = load_malvin_config(input.artifacts.work_dir.as_path())
        .default_workflow
        .max_hypotheses_or_default();
    let want = if want == 0 { DEFAULT_MAX_HYPOTHESES } else { want };
    let groups_block = requirements.groups_block();
    let prompt = router_flow_prompt::build_router_kpop_group_prompt(
        router_flow_prompt::RouterKpopGroupPromptInput {
            store: input.prompt_store,
            artifacts: input.artifacts,
            model: &input.shared.model,
            git: input.shared.git,
            groups_block: &groups_block,
            want,
            exp_log: &exp_log,
        },
    )?;
    run_router_kpop_group_coder_prompt(input.client, &prompt, log_path).await
}

#[cfg(test)]
#[path = "router_flow_acp_support_tests.rs"]
mod router_flow_acp_support_tests;

#[cfg(test)]
#[path = "router_flow_acp_support_kiss_cov_tests.rs"]
mod router_flow_acp_support_kiss_cov_tests;
