use crate::acp::AgentIoOptions;
use crate::artifacts::{
    MalvinChecksBackup, RunArtifacts, SessionDotfileBackups, create_run_artifacts_from_text,
};
use crate::cursor_sdk::CursorSdkClient;
use crate::orchestrator::workflow_context_paths_only;
use crate::prompt_stratification::WorkflowRenderContext;
use crate::prompts::PromptStore;

#[must_use]
pub fn io_opts() -> AgentIoOptions {
    AgentIoOptions {
        force: false,
        no_tee: true,
        raw_output: true,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    }
}

#[must_use]
pub fn no_session_client() -> CursorSdkClient {
    crate::cursor_sdk::cursor_sdk_client_from_raw("cursor:auto", io_opts(), 1)
}

#[must_use]
pub fn empty_dotfile_backups() -> SessionDotfileBackups {
    SessionDotfileBackups::from_parts(crate::session_dotfile_backup::SessionDotfileParts {
        malvin_checks: MalvinChecksBackup::Missing,
        gitignore: crate::session_dotfile_backup::GitignoreBackup::Missing,
        vision: crate::session_dotfile_backup::VisionBackup::Missing,
        malvin_config_workspace:
            crate::session_dotfile_backup::MalvinConfigWorkspaceBackup::Missing,
    })
}

pub fn workflow_ctx_for_smoke(
    tmp: &tempfile::TempDir,
    run_artifact_body: &str,
) -> (RunArtifacts, PromptStore, WorkflowRenderContext) {
    if crate::git_worktree_toplevel(tmp.path()).is_none() {
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(tmp.path())
            .status()
            .expect("git init");
    }
    crate::seed_malvin_checks(tmp.path(), "true\n");
    let artifacts =
        create_run_artifacts_from_text(run_artifact_body, Some(tmp.path())).expect("art");
    let store = PromptStore::default_store();
    let ctx = workflow_context_paths_only(&artifacts, crate::config::DEFAULT_CLI_MODEL, false);
    (artifacts, store, ctx)
}

#[cfg(test)]
mod tests {
    use super::{empty_dotfile_backups, io_opts, no_session_client, workflow_ctx_for_smoke};

    #[test]
    fn io_opts_disables_tee_and_markdown() {
        let o = io_opts();
        assert!(o.no_tee);
        assert!(!o.emit_stdout_markdown);
    }

    #[test]
    fn no_session_client_and_empty_backups_smoke() {
        let _ = no_session_client();
        let backups = empty_dotfile_backups();
        assert!(matches!(
            backups.malvin_checks,
            crate::artifacts::MalvinChecksBackup::Missing
        ));
    }

    #[test]
    fn workflow_ctx_for_smoke_builds_context() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (_artifacts, _store, ctx) = workflow_ctx_for_smoke(&tmp, "support_smoke");
        assert!(ctx.contains_key("plan_path"));
    }
}
