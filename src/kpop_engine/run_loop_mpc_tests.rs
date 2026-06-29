use crate::kpop_engine::KPopHardConstraints;

use super::super::run_loop_exit::{GateLoopExitCtx, kpop_solved_early_exit};
use super::run_loop_tests::gate_early_exit_fixture;
use super::{mpc_done_early_exit, KPopEngineEarlyExitCtx, KpopEngineLoopIterationCtx};

#[test]
fn kpop_solved_early_exit_suppressed_when_mpc_enabled() {
    let (_tmp, artifacts, backups, _bin, _guard) = gate_early_exit_fixture();
    let gate_ctx = GateLoopExitCtx {
        behavior: KPopHardConstraints::CODE,
        artifacts: &artifacts,
        session_dotfile_backups: &backups,
        mpc_enabled: true,
    };
    assert!(!kpop_solved_early_exit(&gate_ctx, 2));
}

#[test]
fn mpc_done_early_exit_requires_marker_and_gates() {
    let (tmp, artifacts, backups, _bin, _guard) = gate_early_exit_fixture();
    let brief = tmp.path().join("brief.md");
    std::fs::write(&brief, "no marker\n").expect("write");
    let code_ctx = GateLoopExitCtx {
        behavior: KPopHardConstraints::CODE,
        artifacts: &artifacts,
        session_dotfile_backups: &backups,
        mpc_enabled: true,
    };
    assert!(!mpc_done_early_exit(&code_ctx, &brief).expect("read"));
    std::fs::write(&brief, "## MPC_DONE\n").expect("write");
    assert!(mpc_done_early_exit(&code_ctx, &brief).expect("read"));
    let delight_ctx = GateLoopExitCtx {
        behavior: KPopHardConstraints::DELIGHT,
        artifacts: &artifacts,
        session_dotfile_backups: &backups,
        mpc_enabled: true,
    };
    assert!(mpc_done_early_exit(&delight_ctx, &brief).expect("read"));
}

#[test]
fn kpop_engine_loop_ctx_types_are_constructible() {
    use std::sync::{Arc, Mutex};

    use crate::artifacts::SessionDotfileBackups;
    use crate::kpop_engine::{KPopEnginePrepared, KPopHardConstraints};
    use super::super::params::KPopEngineParams;

    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
    let store = crate::prompts::PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    let prepared = KPopEnginePrepared {
        artifacts,
        context: crate::prompt_stratification::WorkflowRenderContext::default(),
        request_text: "req".into(),
        startup_emit_request: "req".into(),
        store,
        malvin_checks_backup: crate::artifacts::MalvinChecksBackup::Missing,
    };
    let shared = crate::cli::SharedOpts {
        model: crate::config::DEFAULT_CLI_MODEL.into(),
        no_force: false,
        no_tenacious: false,
        no_tee: true,
        no_markdown: false,
        verbose: false,
        max_acp_retries: 1,
        doc: false,
        name: None,
        mini: false,
        mini_max_bash_turns: 32,
        mini_max_http_turns: 32,
        mini_max_bash_execs: 128,
        mini_max_http_retries: 0,
        mini_max_transport_retries: crate::support_paths::DEFAULT_MAX_MINI_TRANSPORT_RETRIES,
        mini_max_gate_retries: 0,
        mini_max_shrink_passes: 0,
    };
    let params = KPopEngineParams {
        command: "code",
        shared: &shared,
        workflow: crate::cli::WorkflowCliOptions { force: false },
        prepared: &prepared,
        max_loops: 1,
        max_hypotheses: 1,
        behavior: KPopHardConstraints::CODE,
    };
    let run_timing = Arc::new(Mutex::new(crate::run_timing::RunTiming::default()));
    let backups = SessionDotfileBackups::snapshot(tmp.path()).expect("snapshot");
    let _ = KpopEngineLoopIterationCtx {
        params: &params,
        iteration: 1,
        run_timing: &run_timing,
        consecutive_solved: 0,
        mpc_on: true,
    };
    let _ = KPopEngineEarlyExitCtx {
        behavior: KPopHardConstraints::CODE,
        consecutive_solved: 1,
        artifacts: &prepared.artifacts,
        session_dotfile_backups: &backups,
        agent_ran: true,
        run_timing: Some(&run_timing),
        mpc_enabled: false,
    };
}
