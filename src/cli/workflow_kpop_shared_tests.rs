use crate::cli::workflow_kpop_shared::*;

fn kpop_render_fixture(workflow: &str) -> (crate::prompts::PromptStore, crate::artifacts::RunArtifacts) {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
    std::fs::write(tmp.path().join(".malvin/checks"), "kiss check\n").expect("checks");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts(workflow, Some(tmp.path())).expect("artifacts");
    let store = crate::prompts::PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    (store, artifacts)
}

#[test]
fn effective_max_loops_is_at_least_one() {
    assert_eq!(effective_max_loops(0), 1);
    assert_eq!(effective_max_loops(3), 3);
}

#[test]
fn gate_kpop_loop_iterations_is_one_plus_max_loops() {
    assert_eq!(gate_kpop_loop_iterations(0), 2);
    assert_eq!(gate_kpop_loop_iterations(5), 6);
}

#[test]
fn kpop_workflow_context_includes_quality_gates() {
    let (_store, artifacts) = kpop_render_fixture("code");
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
    assert!(err.contains("kiss"));
    let log = std::fs::read_to_string(artifacts.quality_gates_log_path()).expect("read");
    assert!(log.contains("Running `kiss`"));
    assert!(log.contains("[stdout]"));
    assert!(!log.contains("stale output"));
}

#[test]
fn gate_iteration_context_overrides_exp_log() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
    let base = kpop_workflow_context(&artifacts, "code").expect("ctx");
    let iter_log = artifacts.gate_exp_log_path(2);
    let ctx = gate_iteration_context(&base, &artifacts, &iter_log, 2);
    let exp = ctx.get("exp_log").expect("exp_log");
    assert!(exp.contains("_g2.md"));
}

#[test]
fn run_kpop_workspace_gates_restores_before_executing_checks() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (_bin, _guard) = crate::test_agent_client::write_fake_gate(tmp.path(), "kiss", 0);
    let (artifacts, backups) = kpop_gates_restore_fixture(tmp.path());
    std::fs::write(tmp.path().join(".malvin/checks"), "false\n").expect("tamper");
    run_kpop_workspace_gates(&artifacts, &backups, true).expect("gates pass after restore");
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

#[test]
fn restore_failure_prevents_gate_run() {
    use crate::session_dotfile_backup::DotfileBackupState;

    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
    std::fs::write(tmp.path().join(".malvin/checks"), "kiss check\n").expect("checks");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
    let backups = crate::artifacts::SessionDotfileBackups::from_parts(
        crate::artifacts::SessionDotfileParts {
            kissconfig: DotfileBackupState::Present(tmp.path().join("nonexistent-kissconfig")),
            malvin_checks: DotfileBackupState::Missing,
            kissignore: DotfileBackupState::Missing,
            malvin_config: DotfileBackupState::Missing,
            gitignore: DotfileBackupState::Missing,
            malvin_config_workspace: DotfileBackupState::Missing,
        },
    );
    let err = run_kpop_workspace_gates(&artifacts, &backups, true).expect_err("restore fails");
    assert!(err.contains("kissconfig restore"));
}

#[test]
fn render_kpop_program_request_includes_scope() {
    let (store, artifacts) = kpop_render_fixture("code");
    let mut ctx = std::collections::HashMap::new();
    ctx.insert("plan_path".to_string(), "./plan.md".into());
    let text = render_kpop_program_request(&store, "code_constraints.md", &ctx, &artifacts).expect("render");
    assert!(text.contains("quality_gates"));
}
