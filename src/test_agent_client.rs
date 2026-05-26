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
pub fn tidy_test_session(label: &str) -> TidyTestSession {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, label).expect("write plan");
    let artifacts =
        crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    let workflow = crate::cli::WorkflowCliOptions {
        force: false,
        run_learn: false,
    };
    let store = crate::cli::tidy_flow::prepare_tidy_kpop_prompt_store(workflow).expect("store");
    let mut context = crate::workflow_context::workflow_context_paths_only(&artifacts, "tidy");
    context.insert(
        "quality_gates".to_string(),
        crate::repo_gates::prompt_quality_gates_markdown_ephemeral(tmp.path())
            .expect("gates"),
    );
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
pub fn write_fake_gate(
    work_dir: &std::path::Path,
    gate_name: &str,
    exit_code: i32,
) -> (tempfile::TempDir, crate::repo_checks::FakeCommandDirGuard) {
    std::fs::create_dir_all(work_dir.join(crate::MALVIN_DIR)).expect("mkdir .malvin");
    std::fs::write(crate::malvin_checks_path(work_dir), format!("{gate_name}\n"))
        .expect("checks");
    let bin_dir = tempfile::tempdir().expect("bindir");
    install_exit_gate_bin(bin_dir.path(), gate_name, exit_code);
    let guard = crate::repo_checks::set_fake_command_dir(bin_dir.path());
    (bin_dir, guard)
}

#[cfg(test)]
mod write_fake_gate_tests {
    use super::write_fake_gate;

    #[test]
    fn write_fake_gate_seeds_checks_on_workspace_without_malvin_dir() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let work = tmp.path().join("fresh");
        std::fs::create_dir_all(&work).expect("mkdir work");
        let (_bin, _guard) = write_fake_gate(&work, "kiss", 0);
        assert!(work.join(".malvin/checks").is_file());
    }
}
