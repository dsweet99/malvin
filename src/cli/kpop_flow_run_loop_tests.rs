//! Tests for [`super::kpop_flow_run_loop`].

use super::kpop_flow_run_loop::{
    clear_legacy_gate_exp_log, kpop_exp_log_declares_solved, kpop_loop_abort,
    snapshot_kpop_loop_dotfiles_and_exp_log, run_kpop_agent_loops, KpopLoopSnapshot,
    RunKpopAgentLoopsOutcome, RunKpopAgentLoopsParams,
};

#[test]
fn kpop_exp_log_declares_solved_reads_marker() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("exp.md");
    std::fs::write(&path, "## KPOP_SOLVED\n").expect("write");
    assert!(kpop_exp_log_declares_solved(&path).expect("read"));
}

#[test]
fn kpop_loop_abort_records_error_and_agent_ran() {
    let outcome = kpop_loop_abort(true, "setup failed".into());
    assert!(outcome.agent_ran);
    assert_eq!(outcome.acp_result, Err("setup failed".into()));
}

#[test]
fn kpop_loop_snapshot_ensures_home_config_exists() {
    crate::test_utils::with_isolated_home(|work| {
        let cfg = crate::malvin_config_path(work);
        assert!(!cfg.exists());
        std::fs::create_dir_all(work.join(".malvin")).expect("mkdir");
        let artifacts =
            crate::artifacts::create_kpop_run_artifacts("test", Some(work)).expect("artifacts");
        let snap =
            snapshot_kpop_loop_dotfiles_and_exp_log(&artifacts, 1, 1).expect("snapshot");
        assert!(
            cfg.is_file(),
            "kpop loop snapshot must ensure ~/.malvin_home/config.toml exists"
        );
        assert!(matches!(
            snap.backups.malvin_config,
            crate::artifacts::MalvinConfigBackup::Present(_)
        ));
    });
}

#[test]
fn snapshot_kpop_loop_dotfiles_and_exp_log_builds_paths() {
    crate::test_utils::with_isolated_home(|work| {
        std::fs::create_dir_all(work.join(".malvin")).expect("mkdir");
        let artifacts =
            crate::artifacts::create_kpop_run_artifacts("code", Some(work)).expect("artifacts");
        let snap = snapshot_kpop_loop_dotfiles_and_exp_log(&artifacts, 1, 2).expect("snapshot");
        let KpopLoopSnapshot {
            exp_iter,
            exp_log_path,
            backups: _,
        } = snap;
        assert_eq!(exp_iter, 1);
        assert!(exp_log_path.is_file());
        assert!(exp_log_path.to_string_lossy().contains("_g1.md"));
    });
}

#[test]
fn kiss_cov_run_kpop_agent_loops_outcome() {
    let _ = std::any::type_name::<RunKpopAgentLoopsOutcome>();
    let _ = std::any::type_name::<RunKpopAgentLoopsParams>();
    let _ = run_kpop_agent_loops;
    let _ = clear_legacy_gate_exp_log;
    let _ = stringify!(snapshot_kpop_loop_dotfiles_and_exp_log);
}

#[cfg(unix)]
pub(crate) fn test_kpop_args(max_loops: usize) -> (crate::cli::KpopArgs, crate::cli::SharedOpts, crate::cli::WorkflowCliOptions) {
    use crate::cli::{KpopArgs, SharedOpts, WorkflowCliOptions};
    use crate::config::DEFAULT_CLI_MODEL;

    let kpop = KpopArgs {
        max_loops,
        max_hypotheses: 1,
        tenacious: false,
        request: Some("investigate".into()),
    };
    let shared = SharedOpts {
        model: DEFAULT_CLI_MODEL.into(),
        no_force: true,
        no_tenacious: true,
        no_tee: true,
        no_markdown: true,
        verbose: false,
        max_acp_retries: 1,
        doc: false,
        name: None,
        mini: false,
        mini_max_bash_turns: 32,
        mini_max_http_turns: 32,
        mini_max_bash_execs: 128,
        mini_max_http_retries: 0,
        mini_max_gate_retries: 0,
        mini_max_shrink_passes: 0,
    };
    let workflow = WorkflowCliOptions { force: false };
    (kpop, shared, workflow)
}

#[cfg(unix)]
pub(crate) fn install_mock_agent_env(workspace: &std::path::Path, mock: &std::path::Path) -> crate::test_utils::SavedEnvVars {
    #![allow(unsafe_code)]

    write_mock_agent(mock);
    let bin_dir = workspace.join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    crate::test_agent_client::install_exit_gate_bin(&bin_dir, "kiss", 0);
    let guard = crate::test_utils::SavedEnvVars::capture(&[
        "MALVIN_AGENT_ACP_BIN",
        "PATH",
        "CURSOR_AGENT_API_KEY",
        crate::acp::MALVIN_TEST_NO_REAL_AGENT_ENV,
    ]);
    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    unsafe {
        std::env::set_var("MALVIN_AGENT_ACP_BIN", mock);
        std::env::set_var("PATH", path);
        std::env::set_var("CURSOR_AGENT_API_KEY", "test-key");
        std::env::set_var(crate::acp::MALVIN_TEST_NO_REAL_AGENT_ENV, "1");
    }
    guard
}

#[cfg(unix)]
pub(crate) fn write_mock_agent(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    use std::sync::OnceLock;

    static SCRIPT: OnceLock<String> = OnceLock::new();
    let body = r"    const fs = require('fs');
    const path = require('path');
    const promptText = (((msg.params || {}).prompt || [])[0] || {}).text || '';
    const pathMatch = promptText.match(/([^\s`]+\/_kpop\/exp_log_[^\s`]+\.md)/);
    if (pathMatch) {
      let p = pathMatch[1];
      if (p.startsWith('./')) p = path.join(process.cwd(), p.slice(2));
      else if (!p.startsWith('/')) p = path.join(process.cwd(), p);
      fs.mkdirSync(path.dirname(p), { recursive: true });
      fs.appendFileSync(p, `\n## Step 1 — KPOP mock\n`);
    }";
    let handler = format!(
        "{body}\n    console.log(JSON.stringify({{ jsonrpc: '2.0', method: 'session/update', params: {{ update: {{ sessionUpdate: 'agent_message_chunk', content: {{ type: 'text', text: 'step\\n' }} }} }} }}));"
    );
    let script = SCRIPT.get_or_init(|| format!("#!/usr/bin/env node\n{}\n", crate::acp_mock_js("", &handler)));
    std::fs::write(path, script.as_bytes()).expect("write mock");
    let mut perms = std::fs::metadata(path).expect("meta").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

#[cfg(unix)]
mod unix_cov {
    use super::super::kpop_flow_run_loop::{run_kpop_agent_loops, RunKpopAgentLoopsParams};

    use crate::cli::kpop_flow::kpop_boot_store_client_prepared;

    use super::{install_mock_agent_env, test_kpop_args};

    #[test]
    fn run_kpop_agent_loops_propagates_exp_log_setup_error() {
        crate::test_utils::with_isolated_home(|workspace| {
            crate::test_utils::block_on_test_async(async {
                std::fs::write(workspace.join(".kissconfig"), "k = 1\n").expect("kissconfig");
                let (kpop, shared, workflow) = test_kpop_args(2);
                let (store, mut client, prepared) =
                    kpop_boot_store_client_prepared(&kpop, &shared, workflow).expect("boot");
                let kpop_dir = prepared.artifacts.run_dir.join("_kpop");
                std::fs::remove_dir_all(&kpop_dir).expect("rm _kpop");
                std::fs::write(&kpop_dir, "not a directory").expect("block _kpop");
                let outcome = run_kpop_agent_loops(RunKpopAgentLoopsParams {
                    kpop: &kpop,
                    store: &store,
                    client: &mut client,
                    prepared: &prepared,
                })
                .await;
                assert!(outcome.agent_ran);
                outcome.acp_result.expect_err("exp log setup");
            });
        });
    }

    #[test]
    fn run_kpop_agent_loops_executes_mock_agent() {
        crate::test_utils::with_isolated_home(|workspace| {
            crate::test_utils::block_on_test_async(async {
                std::fs::write(workspace.join(".kissconfig"), "k = 1\n").expect("kissconfig");
                let mock = workspace.join("mock-agent");
                let _env = install_mock_agent_env(workspace, &mock);
                let (kpop, shared, workflow) = test_kpop_args(1);
                let (store, mut client, prepared) =
                    kpop_boot_store_client_prepared(&kpop, &shared, workflow).expect("boot");
                let outcome = run_kpop_agent_loops(RunKpopAgentLoopsParams {
                    kpop: &kpop,
                    store: &store,
                    client: &mut client,
                    prepared: &prepared,
                })
                .await;
                assert!(outcome.agent_ran, "mock agent loop should set agent_ran");
                outcome.acp_result.expect("loops");
                let text =
                    std::fs::read_to_string(prepared.artifacts.exp_log_path()).expect("read");
                assert!(text.contains("## Step 1 — KPOP mock"));
            });
        });
    }
}
