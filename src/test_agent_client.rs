//! Shared test doubles for ACP client construction and fake gate binaries.

#[cfg(test)]
pub fn smoke_agent_client() -> crate::acp::AgentClient {
    use crate::acp::{AgentClient, AgentIoOptions};
    AgentClient::new(
        "m".into(),
        AgentIoOptions {
            force: false,
            no_tee: true,
            raw_output: true,
            show_thoughts_on_stdout: false,
            emit_stdout_markdown: false,
            log_full_outgoing_prompts: false,
        },
    )
}

#[cfg(test)]
pub fn install_exit_gate_bin(bin_dir: &std::path::Path, name: &str, code: i32) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let path = bin_dir.join(name);
        std::fs::write(&path, format!("#!/bin/sh\nexit {code}\n")).expect("write fake bin");
        let mut perms = std::fs::metadata(&path).expect("bin meta").permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&path, perms).expect("chmod fake bin");
    }
    #[cfg(windows)]
    {
        let path = bin_dir.join(format!("{name}.cmd"));
        std::fs::write(&path, format!("@exit {code}\r\n")).expect("write fake bin");
    }
    #[cfg(not(any(unix, windows)))]
    {
        let path = bin_dir.join(name);
        std::fs::write(&path, format!("#!/bin/sh\nexit {code}\n")).expect("write fake bin");
    }
}

#[cfg(test)]
pub struct TidyTestSession {
    pub tmp_dir: tempfile::TempDir,
    pub client: crate::acp::AgentClient,
    pub artifacts: crate::artifacts::RunArtifacts,
    pub store: crate::prompts::PromptStore,
    pub context: std::collections::HashMap<String, String>,
    pub backups: crate::artifacts::SessionDotfileBackups,
}

#[cfg(test)]
impl TidyTestSession {
    #[must_use]
    pub(crate) fn recovery_paths(&self) -> crate::cli::tidy_flow::recovery::TidyRecoveryPaths {
        crate::cli::tidy_flow::recovery::TidyRecoveryPaths {
            work_dir: self.artifacts.work_dir.clone(),
            run_dir: self.artifacts.run_dir.clone(),
        }
    }
}

#[cfg(test)]
pub fn tidy_test_session(label: &str) -> TidyTestSession {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, label).expect("write plan");
    let artifacts =
        crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    let store = crate::cli::tidy_flow::prepare_tidy_prompt_store().expect("store");
    let context = crate::workflow_context::workflow_context_paths_only(&artifacts, "tidy");
    let backups = crate::test_utils::empty_session_dotfile_backups(&artifacts.work_dir);
    TidyTestSession {
        tmp_dir: tmp,
        client: smoke_agent_client(),
        artifacts,
        store,
        context,
        backups,
    }
}

#[cfg(test)]
pub fn tidy_acp_input_parts<'a>(
    client: &'a mut crate::acp::AgentClient,
    artifacts: &'a crate::artifacts::RunArtifacts,
    store: &'a crate::prompts::PromptStore,
    context: &'a std::collections::HashMap<String, String>,
) -> crate::cli::tidy_flow::TidyAcpInput<'a> {
    crate::cli::tidy_flow::TidyAcpInput {
        client,
        artifacts,
        store,
        context,
        run_learn: false,
    }
}

#[cfg(test)]
pub fn tidy_acp_input<'a>(session: &'a mut TidyTestSession) -> crate::cli::tidy_flow::TidyAcpInput<'a> {
    tidy_acp_input_parts(
        &mut session.client,
        &session.artifacts,
        &session.store,
        &session.context,
    )
}

#[cfg(test)]
pub fn write_fake_gate(
    work_dir: &std::path::Path,
    gate_name: &str,
    exit_code: i32,
) -> (tempfile::TempDir, crate::repo_checks::FakeCommandDirGuard) {
    std::fs::write(work_dir.join(".malvin_checks"), format!("{gate_name}\n")).expect("checks");
    let bin_dir = tempfile::tempdir().expect("bindir");
    install_exit_gate_bin(bin_dir.path(), gate_name, exit_code);
    let guard = crate::repo_checks::set_fake_command_dir(bin_dir.path());
    (bin_dir, guard)
}
