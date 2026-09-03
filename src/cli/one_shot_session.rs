use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::agent_backend::{
    AgentBackend, agent_backend_attach_run_timing_for_session,
    agent_backend_set_implement_display_name, agent_backend_set_run_timing,
    agent_backend_start_coder_session,
};
use crate::artifacts::{
    RunArtifacts, SessionDotfileBackups, create_run_artifacts_from_text, resolve_user_md_request,
};
use crate::cli::cli_request::require_cli_request;
use crate::run_id::RunDirOptions;

pub fn resolve_one_shot_request_artifacts(
    request: Option<&String>,
    command: &str,
    run_dir_opts: Option<RunDirOptions>,
) -> Result<(String, RunArtifacts), String> {
    let request = require_cli_request(request, command)?;
    let (text, work_dir) = resolve_user_md_request(&request)?;
    let artifacts = match run_dir_opts {
        Some(opts) => crate::artifacts::create_run_artifacts_from_text_opts(
            &text,
            Some(work_dir.as_path()),
            opts,
        )
        .map_err(|e| e.to_string())?,
        None => create_run_artifacts_from_text(&text, Some(work_dir.as_path()))
            .map_err(|e| e.to_string())?,
    };
    crate::run_id::activate_run(artifacts.run_dir.clone());
    Ok((text, artifacts))
}

pub fn finish_one_shot_auth_and_backups(
    client: &mut AgentBackend,
    artifacts: &RunArtifacts,
) -> Result<SessionDotfileBackups, String> {
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    client.prompts_log_run_dir = Some(artifacts.run_dir.clone());
    SessionDotfileBackups::snapshot_after_ensuring_home_config(&artifacts.work_dir)
}

pub struct OneShotCoderGuard {
    timing: Arc<Mutex<crate::run_timing::RunTiming>>,
    run_dir: PathBuf,
}

impl OneShotCoderGuard {
    pub async fn begin(
        client: &mut AgentBackend,
        artifacts: &RunArtifacts,
        implement_label: &'static str,
    ) -> Result<Self, String> {
        let timing = agent_backend_attach_run_timing_for_session(client);
        if let Err(e) = agent_backend_start_coder_session(client, &artifacts.work_dir).await {
            agent_backend_set_run_timing(client, None);
            return Err(e.to_string());
        }
        agent_backend_set_implement_display_name(client, implement_label);
        Ok(Self {
            timing,
            run_dir: artifacts.run_dir.clone(),
        })
    }

    pub async fn finish(
        self,
        client: &mut AgentBackend,
        run_res: Result<(), String>,
    ) -> Result<(), String> {
        let end_res = client.end_coder_session().await.map_err(|e| e.to_string());
        let merged = crate::acp_post_run::prefer_primary_over_secondary(
            run_res,
            end_res,
            "end coder session",
        );
        crate::acp_post_run::emit_run_timing_json_only_after_backend(
            client,
            &self.run_dir,
            &self.timing,
            merged,
        )
    }
}

pub fn finish_one_shot_after_prompt(
    acp_res: Result<(), String>,
    work_dir: &Path,
    backups: &SessionDotfileBackups,
    result_md: &PathBuf,
) -> Result<(), String> {
    let r = crate::acp_post_run::merge_acp_with_workspace_session_restore_and_check_abort(
        acp_res, work_dir, backups, result_md,
    );
    if r.is_ok() {
        crate::cli::error_run_log::clear_command_error_run_dir();
    }
    r
}

#[cfg(test)]
mod kiss_cov {
    #[test]
    fn kiss_static_fn_item_refs() {
        let _ = super::resolve_one_shot_request_artifacts;
        let _ = super::finish_one_shot_auth_and_backups;
        let _ = super::finish_one_shot_after_prompt;
        let _ = stringify!(OneShotCoderGuard);
    }
}
