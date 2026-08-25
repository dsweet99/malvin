use crate::cli::workflow_router_shared::*;
use crate::{
    gate_iteration_context, router_workflow_context, router_workflow_context_without_gates,
    write_checks_do_not_pass_for_artifacts,
};
pub(crate) fn router_render_fixture(
    workflow: &str,
) -> (
    tempfile::TempDir,
    crate::prompts::PromptStore,
    crate::artifacts::RunArtifacts,
) {
    let tmp = tempfile::tempdir().expect("tempdir");
    crate::seed_malvin_checks(tmp.path(), "true\n");
    let artifacts = crate::artifacts::create_run_artifacts_from_text(workflow, Some(tmp.path()))
        .expect("artifacts");
    let store = crate::prompts::PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    (tmp, store, artifacts)
}
fn effective_max_loops_is_at_least_one() {
    assert_eq!(effective_max_loops(0), 1);
    assert_eq!(effective_max_loops(3), 3);
}
fn router_workflow_context_includes_quality_gates() {
    crate::test_utils::with_isolated_home(|_| {
        let (_tmp, _store, artifacts) = router_render_fixture("code");
        let ctx = router_workflow_context!(&artifacts, crate::config::DEFAULT_CLI_MODEL, false)
            .expect("context");
        assert!(ctx.contains_key("quality_gates"));
    });
}
fn router_workflow_context_without_gates_omits_quality_gates() {
    crate::test_utils::with_isolated_home(|_| {
        let (_tmp, _store, artifacts) = router_render_fixture("code");
        let ctx = router_workflow_context_without_gates!(
            &artifacts,
            crate::config::DEFAULT_CLI_MODEL,
            false,
        )
        .expect("context");
        assert!(!ctx.contains_key("quality_gates"));
    });
}
fn prefer_gate_outcome_over_summarize_keeps_gate_error() {
    let err =
        prefer_gate_outcome_over_summarize::<()>(Err("gate boom".into()), Ok(())).unwrap_err();
    assert!(err.contains("gate boom"));
}
fn prefer_gate_outcome_over_summarize_surfaces_summarize_when_gate_ok() {
    let err =
        prefer_gate_outcome_over_summarize(Ok("ok"), Err("summarize boom".into())).unwrap_err();
    assert!(err.contains("summarize boom"));
    let ok = prefer_gate_outcome_over_summarize(Ok(7), Ok(())).expect("ok");
    assert_eq!(ok, 7);
}
fn write_checks_do_not_pass_for_artifacts_writes_markers() {
    crate::test_utils::with_isolated_home(|_| {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts = crate::artifacts::create_run_artifacts_from_text("tidy", Some(tmp.path()))
            .expect("artifacts");
        let workspace_review = tmp.path().join("review.md");
        write_checks_do_not_pass_for_artifacts!(&artifacts).expect("write");
        assert!(artifacts.artifact_review_md().exists());
        assert!(
            !workspace_review.exists(),
            "gate-failure marker must not be written to workspace ./review.md"
        );
    });
}
fn clear_quality_gates_log_for_next_agent_empties_file() {
    crate::test_utils::with_isolated_home(|_| {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts = crate::artifacts::create_run_artifacts_from_text("code", Some(tmp.path()))
            .expect("artifacts");
        let qlog = artifacts.quality_gates_log_path();
        std::fs::write(&qlog, "stale output").expect("write");
        clear_quality_gates_log_for_next_agent(&artifacts).expect("clear");
        assert_eq!(std::fs::read_to_string(&qlog).expect("read"), "");
    });
}
pub(crate) fn gate_failure_fixture(
    exit_code: i32,
) -> (
    tempfile::TempDir,
    tempfile::TempDir,
    crate::repo_checks::FakeCommandDirGuard,
    crate::artifacts::RunArtifacts,
    crate::artifacts::SessionDotfileBackups,
) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (bin, guard) = crate::test_agent_client::write_fake_gate(tmp.path(), "false", exit_code);
    std::fs::write(crate::malvin_checks_path(tmp.path()), "false\n").expect("checks");
    let artifacts = crate::artifacts::create_run_artifacts_from_text("tidy", Some(tmp.path()))
        .expect("artifacts");
    let backups = crate::artifacts::SessionDotfileBackups::snapshot(tmp.path()).expect("snapshot");
    (tmp, bin, guard, artifacts, backups)
}
fn run_router_workspace_gates_refreshes_quality_gates_log() {
    crate::test_utils::with_isolated_home(|_| {
        let (_tmp, _bin, _guard, artifacts, backups) = gate_failure_fixture(1);
        std::fs::write(artifacts.quality_gates_log_path(), "stale output").expect("write");
        let err = run_router_workspace_gates(&artifacts, &backups, true).expect_err("gates fail");
        assert!(
            crate::repo_checks::is_gate_failure_error(&err),
            "gate failure must survive post-gate restore: {err}"
        );
        assert!(err.contains("false"), "expected false gate failure: {err}");
        let log = std::fs::read_to_string(artifacts.quality_gates_log_path()).expect("read");
        assert!(
            log.contains("Running `false`"),
            "expected false gate in log: {log}"
        );
        assert!(log.contains("[stdout]"));
        assert!(!log.contains("stale output"));
    });
}

fn failed_gate_run_does_not_set_just_ran_flag() {
    crate::test_utils::with_isolated_home(|_| {
        let (_tmp, _bin, _guard, artifacts, backups) = gate_failure_fixture(1);
        run_router_workspace_gates(&artifacts, &backups, true).expect_err("gates fail");
        assert!(
            crate::gate_loop_session::quality_gates_just_ran(),
            "completed-but-failed gate run captured output; just_ran must be set"
        );

        // A setup error (nothing executed) must leave the flag off.
        clear_quality_gates_log_for_next_agent(&artifacts).expect("clear");
        let tmp = tempfile::tempdir().expect("tempdir");
        let (artifacts2, backups2) = missing_checks_fixture(tmp.path());
        run_router_workspace_gates(&artifacts2, &backups2, true).expect_err("missing checks fail");
        assert!(
            !crate::gate_loop_session::quality_gates_just_ran(),
            "setup failure (nothing ran) must not set just_ran"
        );
    });
}

fn gate_iteration_context_overrides_exp_log() {
    crate::test_utils::with_isolated_home(|_| {
        let tmp = tempfile::tempdir().expect("tempdir");
        crate::seed_malvin_checks(tmp.path(), "true\n");
        let artifacts = crate::artifacts::create_run_artifacts_from_text("code", Some(tmp.path()))
            .expect("artifacts");
        let base = router_workflow_context!(&artifacts, crate::config::DEFAULT_CLI_MODEL, false)
            .expect("ctx");
        let iter_log = artifacts.gate_exp_log_path(2);
        let ctx = gate_iteration_context!(&base, &artifacts, &iter_log, 2);
        let exp = ctx.get("exp_log").expect("exp_log");
        assert!(exp.contains("_g2.md"));
    });
}
pub(crate) fn missing_checks_fixture(
    work: &std::path::Path,
) -> (
    crate::artifacts::RunArtifacts,
    crate::artifacts::SessionDotfileBackups,
) {
    if crate::git_worktree_toplevel(work).is_none() {
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(work)
            .status()
            .expect("git init");
    }
    let checks_path = crate::malvin_checks_path(work);
    if checks_path.is_file() {
        std::fs::remove_file(&checks_path).expect("remove checks");
    }
    let artifacts =
        crate::artifacts::create_run_artifacts_from_text("code", Some(work)).expect("artifacts");
    let backups = crate::artifacts::SessionDotfileBackups::snapshot(work).expect("snapshot");
    (artifacts, backups)
}
fn run_router_workspace_gates_fails_when_checks_missing() {
    crate::test_utils::with_isolated_home(|_| {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (artifacts, backups) = missing_checks_fixture(tmp.path());
        let err =
            run_router_workspace_gates(&artifacts, &backups, true).expect_err("missing checks");
        assert!(
            err.contains(".malvin/gates is missing"),
            "missing gates must fail clearly: {err}"
        );
    });
}
fn run_router_workspace_gates_restores_before_executing_checks() {
    crate::test_utils::with_isolated_home(|_| {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (_bin, _guard) = crate::test_agent_client::write_fake_gate(tmp.path(), "true", 0);
        let (artifacts, backups) = router_gates_restore_fixture(tmp.path());
        std::fs::write(crate::malvin_checks_path(tmp.path()), "false\n").expect("tamper");
        run_router_workspace_gates(&artifacts, &backups, true).expect("gates pass after restore");
    });
}
fn run_router_workspace_gates_leaves_session_gitignore_after_post_gate_restore() {
    crate::test_utils::with_isolated_home(|_| {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (_bin, _guard) = crate::test_agent_client::write_fake_gate(tmp.path(), "true", 0);
        std::fs::write(tmp.path().join(".gitignore"), "gi\n").expect("drifted gitignore");
        let (artifacts, backups) = router_gates_restore_fixture(tmp.path());
        run_router_workspace_gates(&artifacts, &backups, true).expect("gates pass");
        let gitignore = std::fs::read_to_string(tmp.path().join(".gitignore")).expect("read");
        assert_eq!(
            gitignore, "gi\n",
            "post-gate restore replays session snapshot without reconcile"
        );
    });
}
fn restore_failure_prevents_gate_run() {
    crate::test_utils::with_isolated_home(|_| {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (artifacts, backups) = gitignore_restore_failure_fixture(tmp.path());
        let err =
            run_router_workspace_gates(&artifacts, &backups, true).expect_err("restore fails");
        assert!(err.contains("gitignore restore"));
    });
}
fn prefer_gate_outcome_keeps_gate_error_when_restore_also_fails() {
    let gate = Err("__MALVIN_GATE_FAILURE__:`false` failed (exit 1)".into());
    let restore = Err("gitignore restore: Is a directory".into());
    let err = prefer_gate_outcome_over_post_gate_cleanup(gate, restore).unwrap_err();
    assert!(err.contains("false"));
    assert!(!err.contains("gitignore restore"));
}
#[path = "workflow_router_shared_tests_tail.rs"]
mod workflow_router_shared_tests_tail;
pub(crate) use workflow_router_shared_tests_tail::artifact_storage_available;
pub(crate) use workflow_router_shared_tests_tail::{
    gitignore_restore_failure_fixture, router_gates_restore_fixture,
};

#[test]
fn kiss_bundled_cli_workflow_router_shared_tests() {
    effective_max_loops_is_at_least_one();
    router_workflow_context_includes_quality_gates();
    router_workflow_context_without_gates_omits_quality_gates();
    prefer_gate_outcome_over_summarize_keeps_gate_error();
    prefer_gate_outcome_over_summarize_surfaces_summarize_when_gate_ok();
    write_checks_do_not_pass_for_artifacts_writes_markers();
    clear_quality_gates_log_for_next_agent_empties_file();
    run_router_workspace_gates_refreshes_quality_gates_log();
    failed_gate_run_does_not_set_just_ran_flag();
    gate_iteration_context_overrides_exp_log();
    run_router_workspace_gates_fails_when_checks_missing();
    run_router_workspace_gates_restores_before_executing_checks();
    run_router_workspace_gates_leaves_session_gitignore_after_post_gate_restore();
    restore_failure_prevents_gate_run();
    prefer_gate_outcome_keeps_gate_error_when_restore_also_fails();
}
