//! Shared fixtures for orchestrator unit tests (keeps per-test duplication low for `kiss check`).

use std::collections::HashMap;

use crate::acp::{AgentClient, AgentIoOptions};
use crate::artifacts::{
    KissConfigBackup, KissignoreBackup, MalvinChecksBackup, MalvinConfigBackup, RunArtifacts,
    SessionDotfileBackups, create_run_artifacts_from_text,
};
use crate::orchestrator::workflow_context;
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
pub fn no_session_client() -> AgentClient {
    AgentClient::new("m".into(), io_opts())
}

#[must_use]
pub fn empty_dotfile_backups() -> SessionDotfileBackups {
    SessionDotfileBackups::from_parts(crate::session_dotfile_backup::SessionDotfileParts {
        kissconfig: KissConfigBackup::Missing,
        malvin_checks: MalvinChecksBackup::Missing,
        kissignore: KissignoreBackup::Missing,
        malvin_config: MalvinConfigBackup::Missing,
        gitignore: crate::session_dotfile_backup::GitignoreBackup::Missing,
        vision: crate::session_dotfile_backup::VisionBackup::Missing,
        malvin_config_workspace: crate::session_dotfile_backup::MalvinConfigWorkspaceBackup::Missing,
    })
}

/// Fresh run artifacts, default prompt store, and a `workflow_context` map under `tmp`.
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
    let artifacts =
        create_run_artifacts_from_text(run_artifact_body, Some(tmp.path())).expect("art");
    let store = PromptStore::default_store();
    let ctx = workflow_context(&artifacts, &store, "code").expect("ctx");
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
            backups.kissconfig,
            crate::artifacts::KissConfigBackup::Missing
        ));
    }

    #[test]
    fn workflow_ctx_for_smoke_builds_context() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (_artifacts, _store, ctx) = workflow_ctx_for_smoke(&tmp, "support_smoke");
        assert!(ctx.contains_key("plan_path"));
    }
}
