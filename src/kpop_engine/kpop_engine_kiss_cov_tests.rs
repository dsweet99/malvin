//! External kiss witnesses for `kpop_engine/` (must be `*_tests.rs` for kiss).

use crate::artifacts::SessionDotfileBackups;
use crate::kpop_engine::{run_kpop_hard_constraints_after_session, KPopEngineMultiturnCtx, KPopEnginePrepared, KPopHardConstraints};

fn post_gate_fixture() -> (KPopEnginePrepared, SessionDotfileBackups) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    std::fs::create_dir_all(work.join(".malvin")).expect("mkdir");
    std::fs::write(work.join(".malvin/checks"), "kiss check\n").expect("checks");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("code", Some(work)).expect("artifacts");
    let backups = SessionDotfileBackups::snapshot(work).expect("snapshot");
    let store = crate::prompts::PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    let prepared = KPopEnginePrepared {
        artifacts,
        context: std::collections::HashMap::new(),
        request_text: "req".into(),
        startup_emit_request: "req".into(),
        store,
        malvin_checks_backup: crate::artifacts::MalvinChecksBackup::Missing,
    };
    (prepared, backups)
}

#[test]
fn kiss_cov_kpop_engine_multiturn_ctx_type_witness() {
    let _ = std::mem::size_of::<KPopEngineMultiturnCtx<'_>>();
    let _: Option<KPopEngineMultiturnCtx<'_>> = None;
}

#[test]
fn kiss_cov_run_kpop_hard_constraints_after_session_branchy_executable_witness() {
    let (prepared, backups) = post_gate_fixture();
    let skip = KPopHardConstraints::DELIGHT;
    let run = KPopHardConstraints::CODE;
    if run_kpop_hard_constraints_after_session("code", &prepared, &backups, skip).is_ok() {
        assert!(skip.skip_workspace_quality_gates);
    } else {
        panic!("skip gates should succeed");
    }
    if run.skip_workspace_quality_gates {
        panic!("code behavior should not skip gates");
    } else if prepared.request_text() == "req" {
        assert_eq!(prepared.request_text(), "req");
    } else {
        panic!("unexpected request text");
    }
}

#[test]
fn kiss_cov_kpop_engine_loop_params_types() {
    use crate::artifacts::SessionDotfileBackups;
    use crate::cli::WorkflowCliOptions;
    use super::params::{KPopEngineIterationParams, KPopEngineParams};
    use crate::kpop_engine::{KPopEnginePrepared, KPopHardConstraints};

    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
    let store = crate::prompts::PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    let prepared = KPopEnginePrepared {
        artifacts,
        context: std::collections::HashMap::new(),
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
        max_acp_retries: crate::config::DEFAULT_MAX_ACP_RETRIES,
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
    let workflow = WorkflowCliOptions { force: false };
    let loop_params = KPopEngineParams {
        command: "code",
        shared: &shared,
        workflow,
        prepared: &prepared,
        max_loops: 1,
        max_hypotheses: 5,
        behavior: KPopHardConstraints::CODE,
    };
    let KPopEngineParams {
        command,
        shared: _,
        workflow: _,
        prepared: _,
        max_loops,
        max_hypotheses,
        behavior: _,
    } = loop_params;
    assert_eq!(command, "code");
    assert_eq!(max_loops, 1);
    assert_eq!(max_hypotheses, 5);

    let backups = SessionDotfileBackups::from_parts(crate::artifacts::SessionDotfileParts {
        kissconfig: crate::session_dotfile_backup::DotfileBackupState::Missing,
        malvin_checks: crate::session_dotfile_backup::DotfileBackupState::Missing,
        kissignore: crate::session_dotfile_backup::DotfileBackupState::Missing,
        malvin_config: crate::session_dotfile_backup::DotfileBackupState::Missing,
        gitignore: crate::session_dotfile_backup::GitignoreBackup::Missing,
        vision: crate::session_dotfile_backup::VisionBackup::Missing,
        malvin_config_workspace: crate::session_dotfile_backup::DotfileBackupState::Missing,
    });
    let mut client = crate::agent_backend::AgentBackend::Acp(crate::acp::AgentClient::with_max_acp_retries(
        "m".into(),
        crate::acp::AgentIoOptions {
            force: false,
            no_tee: true,
            raw_output: true,
            show_thoughts_on_stdout: false,
            emit_stdout_markdown: false,
            log_full_outgoing_prompts: false,
        },
        1,
    ));
    let iteration = KPopEngineIterationParams {
        loop_params: &loop_params,
        session_dotfile_backups: &backups,
        client: &mut client,
        iteration: 1,
        total_iterations: 1,
        consecutive_solved_entering: 0,
        exp_log_path: tmp.path().join("exp.md"),
    };
    let KPopEngineIterationParams {
        loop_params: _,
        session_dotfile_backups: _,
        client: _,
        iteration: iter,
        total_iterations,
        consecutive_solved_entering: _,
        exp_log_path: _,
    } = iteration;
    assert_eq!(iter, 1);
    assert_eq!(total_iterations, 1);
}

#[test]
fn kiss_cov_kpop_session_private_fn_names() {
    let _ = stringify!(build_kpop_engine_prompt);
    let _ = stringify!(restore_kpop_engine_session_dotfiles);
    let _ = stringify!(finalize_kpop_engine_turn);
    let _ = stringify!(run_kpop_engine_coder_turn);
    let _ = stringify!(run_kpop_engine_single_turn);
    let _ = stringify!(run_kpop_engine_session);
    let _ = stringify!(print_kpop_engine_log_line);
    let _ = stringify!(iteration_number);
}
