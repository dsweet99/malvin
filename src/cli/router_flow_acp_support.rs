use crate::artifacts::{
    ensure_gate_exp_log_file, GitignoreBackup, MalvinChecksBackup, MalvinConfigBackup,
    MalvinConfigWorkspaceBackup, RunArtifacts, SessionDotfileBackups, VisionBackup,
};
use crate::malvin_config_file::DEFAULT_MAX_HYPOTHESES;
use crate::router_flow::router_flow_parse::{load_review_requirements, ReviewRequirementGroup};
use crate::router_flow::router_flow_prompt;
use std::path::Path;

use super::router_flow_coder_prompts::{
    run_router_kpop_group_coder_prompt, run_router_requirements_coder_prompt,
    run_router_work_coder_prompt,
};
use super::RouterAcpIterationInput;

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
) -> Result<SessionDotfileBackups, String> {
    let work_dir = input.artifacts.work_dir.as_path();
    let iteration_backups = SessionDotfileBackups::snapshot_after_ensuring_home_config(work_dir)?;
    run_router_requirements_coder_prompt(input.client, input.coder, log_path).await?;
    let requirements =
        load_review_requirements(&crate::artifacts::review_requirements_json(input.artifacts))?;
    for (i, group) in requirements.groups.iter().enumerate() {
        run_one_group_kpop(input, log_path, i + 1, group).await?;
    }
    let work_body = router_flow_prompt::build_router_work_prompt(router_flow_prompt::RouterWorkPromptInput {
        store: input.prompt_store,
        artifacts: input.artifacts,
        model: &input.shared.model,
        git: input.shared.git,
        gates: input.shared.gates,
    })?;
    run_router_work_coder_prompt(input.client, &work_body, log_path).await?;
    Ok(iteration_backups)
}

async fn run_one_group_kpop(
    input: &mut RouterAcpIterationInput<'_>,
    log_path: &Path,
    group_index: usize,
    group: &ReviewRequirementGroup,
) -> Result<(), String> {
    let exp_log = ensure_gate_exp_log_file(input.artifacts, group_index).map_err(|e| e.to_string())?;
    let prompt = router_flow_prompt::build_router_kpop_group_prompt(
        router_flow_prompt::RouterKpopGroupPromptInput {
            store: input.prompt_store,
            artifacts: input.artifacts,
            model: &input.shared.model,
            git: input.shared.git,
            group_index,
            group_title: &group.title_trimmed(),
            group_requirements: &group.requirements_bullet_list(),
            want: DEFAULT_MAX_HYPOTHESES,
            exp_log: &exp_log,
        },
    )?;
    run_router_kpop_group_coder_prompt(input.client, &prompt, log_path, group_index).await
}

#[cfg(test)]
#[path = "router_flow_acp_support_tests.rs"]
mod router_flow_acp_support_tests;

#[cfg(test)]
#[path = "router_flow_acp_support_kiss_cov_tests.rs"]
mod router_flow_acp_support_kiss_cov_tests;
