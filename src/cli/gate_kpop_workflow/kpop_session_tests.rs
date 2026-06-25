//! Tests for [`super::kpop_session`].

use super::params::GateKpopIterationParams;
use super::prepared::GateKpopPrepared;
use super::{run_gate_kpop_session, GateKpopLoopParams, GateKpopMultiturnCtx, GateLoopBehavior};
use crate::agent_backend::AgentBackend;
use crate::artifacts::SessionDotfileBackups;
use crate::cli::{SharedOpts, WorkflowCliOptions};
use crate::config::DEFAULT_MAX_ACP_RETRIES;
use std::path::Path;

pub(crate) enum PreparedContextMode {
    Empty,
    /// Path keys only — skips ephemeral quality-gates markdown expansion (faster for session smoke tests).
    PathsOnly,
    Workflow,
}

pub(crate) fn prepared_context_for_mode(
    artifacts: &crate::artifacts::RunArtifacts,
    command: &str,
    context_mode: PreparedContextMode,
) -> std::collections::HashMap<String, String> {
    match context_mode {
        PreparedContextMode::Empty => std::collections::HashMap::new(),
        PreparedContextMode::PathsOnly => {
            crate::cli::workflow_kpop_shared::kpop_workflow_context_without_gates(
                artifacts,
                command,
            )
            .expect("paths context")
        }
        PreparedContextMode::Workflow => crate::cli::workflow_kpop_shared::kpop_workflow_context(
            artifacts,
            command,
        )
        .expect("workflow context"),
    }
}

pub(crate) fn prepared_fixture(
    command: &str,
    work: &Path,
    with_checks: bool,
    context_mode: PreparedContextMode,
) -> (GateKpopPrepared, SessionDotfileBackups) {
    if with_checks {
        std::fs::create_dir_all(work.join(".malvin")).expect("mkdir");
        std::fs::write(work.join(".malvin/checks"), "kiss check\n").expect("checks");
    }
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts(command, Some(work)).expect("artifacts");
    let backups = SessionDotfileBackups::snapshot(work).expect("snapshot");
    let store = crate::prompts::PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    let context = prepared_context_for_mode(&artifacts, command, context_mode);
    let prepared = GateKpopPrepared {
        artifacts,
        context,
        request_text: "req".into(),
        startup_emit_request: "req".into(),
        store,
        malvin_checks_backup: crate::artifacts::MalvinChecksBackup::Missing,
    };
    (prepared, backups)
}

pub(crate) fn shared_workflow() -> (SharedOpts, WorkflowCliOptions) {
    (
        crate::cli::kpop_summarize_tests::summarize_shared_opts(DEFAULT_MAX_ACP_RETRIES),
        WorkflowCliOptions { force: false },
    )
}

pub(crate) fn loop_params<'a>(
    command: &'a str,
    shared: &'a SharedOpts,
    prepared: &'a GateKpopPrepared,
    behavior: GateLoopBehavior,
) -> GateKpopLoopParams<'a> {
    GateKpopLoopParams {
        command,
        shared,
        workflow: WorkflowCliOptions { force: false },
        prepared,
        max_loops: 1,
        max_hypotheses: 1,
        behavior,
    }
}

pub(crate) fn agent_backend(shared: &SharedOpts, command: &str) -> AgentBackend {
    crate::agent_backend::build_agent_backend(
        shared,
        WorkflowCliOptions { force: false },
        false,
        command,
    )
    .expect("backend")
}

pub(crate) struct IterationFixture<'a> {
    pub loop_params: &'a GateKpopLoopParams<'a>,
    pub backups: &'a SessionDotfileBackups,
    pub client: &'a mut AgentBackend,
    pub iteration: usize,
    pub total_iterations: usize,
    pub exp_log_path: std::path::PathBuf,
}

pub(crate) fn build_iteration_params(input: IterationFixture<'_>) -> GateKpopIterationParams<'_> {
    GateKpopIterationParams {
        loop_params: input.loop_params,
        session_dotfile_backups: input.backups,
        client: input.client,
        iteration: input.iteration,
        total_iterations: input.total_iterations,
        consecutive_solved_entering: 0,
        exp_log_path: input.exp_log_path,
    }
}

#[test]
fn gate_kpop_session_declared_solved_detects_kpop_solved_marker() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("exp.md");
    std::fs::write(&path, "## KPOP_SOLVED\n").expect("write");
    assert!(super::run_loop::session_wrote_kpop_solved(&path).expect("read"));
}

#[cfg(unix)]
#[test]
fn kiss_cov_gate_kpop_multiturn_ctx_reads_iteration_field() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    let (_bin, _guard) = crate::test_agent_client::write_fake_gate(work, "agent", 0);
    let (prepared, backups) =
        prepared_fixture("code", work, true, PreparedContextMode::Empty);
    let (shared, _) = shared_workflow();
    let loop_params = loop_params("code", &shared, &prepared, GateLoopBehavior::CODE);
    let mut client = agent_backend(&shared, "code");
    let exp_log_path = prepared.artifacts().gate_exp_log_path(1);
    let mut iteration_params = build_iteration_params(IterationFixture {
        loop_params: &loop_params,
        backups: &backups,
        client: &mut client,
        iteration: 1,
        total_iterations: 2,
        exp_log_path,
    });
    let ctx = GateKpopMultiturnCtx {
        iteration: &mut iteration_params,
    };
    assert_eq!(ctx.iteration.iteration, 1);
    assert_eq!(ctx.iteration.total_iterations, 2);
    assert_eq!(ctx.iteration_number(), 1);
}

#[cfg(unix)]
#[test]
fn kiss_cov_run_gate_kpop_session_success_branch() {
    crate::test_utils::enable_test_fast_teardown();
    crate::test_utils::with_isolated_home(|work| {
        let mock = work.join("mock-gate-kpop-agent");
        let _env =
            crate::cli::kpop_flow::kpop_flow_run_loop_tests::install_mock_agent_env(work, &mock);
        let (prepared, backups) =
            prepared_fixture("code", work, true, PreparedContextMode::PathsOnly);
        let (shared, _) = shared_workflow();
        let loop_params = loop_params("code", &shared, &prepared, GateLoopBehavior::CODE);
        let mut client = agent_backend(&shared, "code");
        let exp_log_path = prepared.artifacts().gate_exp_log_path(1);
        let mut iteration_params = build_iteration_params(IterationFixture {
            loop_params: &loop_params,
            backups: &backups,
            client: &mut client,
            iteration: 1,
            total_iterations: 1,
            exp_log_path,
        });
        let mut ctx = GateKpopMultiturnCtx {
            iteration: &mut iteration_params,
        };
        crate::test_utils::block_on_test_async(async {
            let post = run_gate_kpop_session(&mut ctx).await.expect("successful session");
            let checks_ok = matches!(
                post.malvin_checks,
                crate::session_dotfile_backup::DotfileBackupState::Present(_)
            ) || matches!(
                post.malvin_checks,
                crate::session_dotfile_backup::DotfileBackupState::Missing
            );
            assert!(checks_ok);
            assert_eq!(ctx.iteration_number(), 1);
        });
    });
}
