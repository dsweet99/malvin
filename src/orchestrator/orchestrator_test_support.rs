//! Shared fixtures for orchestrator unit tests (keeps per-test duplication low for `kiss check`).

use std::collections::HashMap;

use crate::acp::{AgentClient, AgentIoOptions};
use crate::artifacts::{
    KissConfigBackup, KissignoreBackup, MalvinChecksBackup, RunArtifacts, SessionDotfileBackups,
    create_run_artifacts_from_text,
};
use crate::orchestrator::workflow_context;
use crate::prompts::PromptStore;

#[must_use]
pub fn io_opts() -> AgentIoOptions {
    AgentIoOptions {
        force: false,
        no_sandbox: true,
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
    SessionDotfileBackups::from_parts(
        KissConfigBackup::Missing,
        MalvinChecksBackup::Missing,
        KissignoreBackup::Missing,
    )
}

/// Fresh run artifacts, default prompt store, and a `workflow_context` map under `tmp`.
pub fn workflow_ctx_for_smoke(
    tmp: &tempfile::TempDir,
    run_artifact_body: &str,
) -> (RunArtifacts, PromptStore, HashMap<String, String>) {
    let artifacts =
        create_run_artifacts_from_text(run_artifact_body, Some(tmp.path())).expect("art");
    let store = PromptStore::default_store();
    let ctx = workflow_context(&artifacts, &store, "code").expect("ctx");
    (artifacts, store, ctx)
}

#[cfg(test)]
mod tests {
    use super::io_opts;

    #[test]
    fn io_opts_disables_tee_and_markdown() {
        let o = io_opts();
        assert!(o.no_tee);
        assert!(!o.emit_stdout_markdown);
    }
}
