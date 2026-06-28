use crate::cli::workflow_kpop_shared::*;
use crate::kpop_program::render_repo_program;

fn kpop_render_fixture(
    workflow: &str,
) -> (
    tempfile::TempDir,
    crate::prompts::PromptStore,
    crate::artifacts::RunArtifacts,
) {
    let tmp = tempfile::tempdir().expect("tempdir");
    crate::seed_malvin_checks(tmp.path(), "kiss check\n");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts(workflow, Some(tmp.path())).expect("artifacts");
    let store = crate::prompts::PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    (tmp, store, artifacts)
}

#[test]
fn effective_max_loops_is_at_least_one() {
    assert_eq!(effective_max_loops(0), 1);
    assert_eq!(effective_max_loops(3), 3);
}

#[test]
fn kpop_engine_loop_iterations_is_one_plus_max_loops() {
    crate::test_utils::clear_test_no_real_agent_env();
    assert_eq!(kpop_engine_loop_iterations(0), 2);
    assert_eq!(kpop_engine_loop_iterations(5), 6);
}

#[test]
fn kpop_workflow_context_includes_quality_gates() {
    let (_tmp, _store, artifacts) = kpop_render_fixture("code");
    let ctx = kpop_workflow_context(&artifacts, "code").expect("context");
    assert!(ctx.contains_key("quality_gates"));
}

#[test]
fn write_checks_do_not_pass_for_artifacts_writes_markers() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("tidy", Some(tmp.path())).expect("artifacts");
    let workspace_review = tmp.path().join("review.md");
    write_checks_do_not_pass_for_artifacts(&artifacts).expect("write");
    assert!(artifacts.artifact_review_md().exists());
    assert!(
        !workspace_review.exists(),
        "gate-failure marker must not be written to workspace ./review.md"
    );
}

#[test]
fn clear_quality_gates_log_for_next_agent_empties_file() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
    let qlog = artifacts.quality_gates_log_path();
    std::fs::write(&qlog, "stale output").expect("write");
    clear_quality_gates_log_for_next_agent(&artifacts).expect("clear");
    assert_eq!(std::fs::read_to_string(&qlog).expect("read"), "");
}

#[test]
fn run_kpop_workspace_gates_refreshes_quality_gates_log() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (_bin, _guard) = crate::test_agent_client::write_fake_gate(tmp.path(), "kiss", 1);
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("tidy", Some(tmp.path())).expect("artifacts");
    std::fs::write(artifacts.quality_gates_log_path(), "stale output").expect("write");
    let backups = crate::artifacts::SessionDotfileBackups::snapshot(tmp.path()).expect("snapshot");
    let err = run_kpop_workspace_gates(&artifacts, &backups, true).expect_err("gates fail");
    assert!(
        crate::repo_checks::is_gate_failure_error(&err),
        "gate failure must survive post-gate restore: {err}"
    );
    assert!(err.contains("kiss"), "expected kiss gate failure: {err}");
    let log = std::fs::read_to_string(artifacts.quality_gates_log_path()).expect("read");
    assert!(
        log.contains("Running `kiss check`"),
        "bare kiss in backups is repaired before gates run: {log}"
    );
    assert!(log.contains("[stdout]"));
    assert!(!log.contains("stale output"));
}

#[test]
fn gate_iteration_context_overrides_exp_log() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(tmp.path())
        .status()
        .expect("git init");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
    let base = kpop_workflow_context(&artifacts, "code").expect("ctx");
    let iter_log = artifacts.gate_exp_log_path(2);
    let ctx = gate_iteration_context(&base, &artifacts, &iter_log, 2);
    let exp = ctx.get("exp_log").expect("exp_log");
    assert!(exp.contains("_g2.md"));
}

fn bare_kiss_repair_fixture(
    work: &std::path::Path,
) -> (
    crate::artifacts::RunArtifacts,
    (
        tempfile::TempDir,
        crate::repo_checks::FakeCommandDirGuard,
    ),
) {
    if crate::git_worktree_toplevel(work).is_none() {
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(work)
            .status()
            .expect("git init");
    }
    std::fs::write(
        work.join(".kissconfig"),
        "[gate]\ntest_coverage_threshold = 0\n",
    )
    .expect("kissconfig");
    let guard = crate::test_agent_client::write_fake_gate(work, "kiss", 0);
    std::fs::write(crate::malvin_checks_path(work), "kiss\n").expect("re-poison");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("code", Some(work)).expect("artifacts");
    (artifacts, guard)
}

#[test]
fn run_kpop_workspace_gates_repairs_bare_kiss_after_poisoned_restore() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (artifacts, (_bin, _guard)) = bare_kiss_repair_fixture(tmp.path());
    let backups = crate::artifacts::SessionDotfileBackups::snapshot(tmp.path()).expect("snapshot");
    run_kpop_workspace_gates(&artifacts, &backups, true).expect("gates pass after repair");
    let log = std::fs::read_to_string(artifacts.quality_gates_log_path()).expect("read log");
    assert!(
        log.contains("Running `kiss check`"),
        "repair must normalize bare kiss before gates: {log}"
    );
}

#[test]
fn run_kpop_workspace_gates_restores_before_executing_checks() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (_bin, _guard) = crate::test_agent_client::write_fake_gate(tmp.path(), "kiss", 0);
    let (artifacts, backups) = kpop_gates_restore_fixture(tmp.path());
    std::fs::write(crate::malvin_checks_path(tmp.path()), "false\n").expect("tamper");
    run_kpop_workspace_gates(&artifacts, &backups, true).expect("gates pass after restore");
}

#[test]
fn run_kpop_workspace_gates_leaves_session_gitignore_after_post_gate_restore() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (_bin, _guard) = crate::test_agent_client::write_fake_gate(tmp.path(), "kiss", 0);
    std::fs::write(tmp.path().join(".gitignore"), "gi\n").expect("drifted gitignore");
    let (artifacts, backups) = kpop_gates_restore_fixture(tmp.path());
    run_kpop_workspace_gates(&artifacts, &backups, true).expect("gates pass");
    let gitignore = std::fs::read_to_string(tmp.path().join(".gitignore")).expect("read");
    assert_eq!(gitignore, "gi\n", "post-gate restore replays session snapshot without reconcile");
}

fn kpop_gates_restore_fixture(
    work: &std::path::Path,
) -> (crate::artifacts::RunArtifacts, crate::artifacts::SessionDotfileBackups) {
    std::fs::create_dir_all(work.join(".malvin")).expect("mkdir");
    std::fs::write(work.join(".malvin/checks"), "kiss check\n").expect("checks");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("code", Some(work)).expect("artifacts");
    let backups = crate::artifacts::SessionDotfileBackups::snapshot(work).expect("snapshot");
    (artifacts, backups)
}

fn kissconfig_restore_failure_fixture(
    work: &std::path::Path,
) -> (crate::artifacts::RunArtifacts, crate::artifacts::SessionDotfileBackups) {
    std::fs::create_dir_all(work.join(".malvin")).expect("mkdir");
    std::fs::write(work.join(".malvin/checks"), "kiss check\n").expect("checks");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("code", Some(work)).expect("artifacts");
    std::fs::write(work.join(".kissconfig"), "orig\n").expect("kissconfig");
    let backups = crate::artifacts::SessionDotfileBackups::snapshot(work).expect("snapshot");
    std::fs::remove_file(work.join(".kissconfig")).expect("remove kissconfig");
    std::fs::create_dir(work.join(".kissconfig")).expect("kissconfig dir");
    (artifacts, backups)
}

#[test]
fn restore_failure_prevents_gate_run() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (artifacts, backups) = kissconfig_restore_failure_fixture(tmp.path());
    let err = run_kpop_workspace_gates(&artifacts, &backups, true).expect_err("restore fails");
    assert!(err.contains("kissconfig restore"));
}

#[test]
fn render_repo_program_includes_scope() {
    let (_tmp, store, artifacts) = kpop_render_fixture("code");
    let mut ctx = std::collections::HashMap::new();
    ctx.insert("plan_path".to_string(), "./plan.md".into());
    let text = render_repo_program(&store, "code_constraints.md", &ctx, &artifacts).expect("render");
    assert!(text.contains("quality_gates"));
}

#[test]
fn prefer_gate_outcome_keeps_gate_error_when_restore_also_fails() {
    let gate = Err("__MALVIN_GATE_FAILURE__:`kiss check` failed (exit 1)".into());
    let restore = Err("gitignore restore: Is a directory".into());
    let err = prefer_gate_outcome_over_post_gate_cleanup(gate, restore).unwrap_err();
    assert!(err.contains("kiss check"));
    assert!(!err.contains("gitignore restore"));
}

#[test]
fn prefer_gate_outcome_surfaces_restore_when_gate_passed() {
    let err = prefer_gate_outcome_over_post_gate_cleanup(
        Ok(()),
        Err("malvin_checks restore: boom".into()),
    )
    .unwrap_err();
    assert!(err.contains("malvin_checks restore"));
}
