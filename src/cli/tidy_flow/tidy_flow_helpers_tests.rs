use std::collections::HashMap;

#[test]
fn effective_tidy_max_loops_zero_means_one() {
    assert_eq!(crate::cli::tidy_flow::effective_tidy_max_loops(0), 1);
}

#[test]
fn tidy_recovery_stdout_line_labels_recovery_not_budget_iteration() {
    assert_eq!(
        crate::cli::tidy_flow::recovery::tidy_recovery_stdout_line(2, 1),
        "tidy recovery (review attempt 2, max-loops 1)"
    );
}

#[test]
fn write_checks_do_not_pass_writes_marker_line() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let p = tmp.path().join("nested").join("review.md");
    crate::cli::tidy_flow::write_checks_do_not_pass_to_review_path(&p).expect("write");
    assert_eq!(
        std::fs::read_to_string(&p).expect("read"),
        "Checks do not pass\n"
    );
}

#[test]
fn gate_failure_marker_not_masked_by_stale_workspace_lgtm_on_sync() {
    use crate::review_sync::{is_lgtm_str, sync_review_file_for_attempt};

    let t = tempfile::tempdir().expect("tempdir");
    let artifacts =
        crate::artifacts::create_run_artifacts_from_text("gate_marker", Some(t.path()))
            .expect("artifacts");
    let artifact = artifacts.artifact_review_md();
    let workspace = artifacts.workspace_review_md();
    std::fs::write(&workspace, "LGTM\n").expect("workspace lgtm");
    crate::cli::tidy_flow::write_checks_do_not_pass_for_artifacts(&artifacts).expect("markers");
    let synced = sync_review_file_for_attempt(&artifact, &workspace).expect("sync");
    assert!(
        synced
            .as_deref()
            .is_some_and(|text| !is_lgtm_str(text) && text.contains("Checks do not pass")),
        "after in-loop gate failure, sync must not treat stale workspace LGTM as authoritative \
         over the artifact gate-failure marker (got {synced:?})"
    );
}

#[test]
fn prepare_tidy_prompt_store_loads_required_templates() {
    let store = crate::cli::tidy_flow::prepare_tidy_prompt_store().expect("store");
    let mut ctx = std::collections::HashMap::new();
    ctx.insert("memories".to_string(), String::new());
    ctx.insert("quality_gates".to_string(), String::new());
    ctx.insert(
        "quality_gates_log".to_string(),
        "./_malvin/run/quality_gates.log".to_string(),
    );
    ctx.insert("plan_path".to_string(), "./plan.md".to_string());
    let prompt = crate::cli::tidy_flow::compose_tidy_prompt(&store, &ctx).expect("compose");
    assert!(prompt.contains("tidy") || prompt.len() > 20);
}

#[test]
fn compose_tidy_concerns_includes_review_path_when_present_in_context() {
    let store = crate::prompts::PromptStore::default_store();
    let mut ctx = HashMap::new();
    ctx.insert("memories".to_string(), String::new());
    ctx.insert(
        "quality_gates_log".to_string(),
        "./_malvin/run/quality_gates.log".to_string(),
    );
    ctx.insert("quality_gates".to_string(), "- `kiss check`\n".to_string());
    ctx.insert("plan_path".to_string(), "./plan.md".to_string());
    ctx.insert(
        "review_path".to_string(),
        "./_malvin/run/review.md".to_string(),
    );
    let out = crate::cli::tidy_flow::compose_tidy_concerns_prompt(&store, &ctx).expect("compose");
    assert!(
        out.contains("./_malvin/run/review.md"),
        "expected rendered concerns to cite review_path: {out:?}"
    );
}

#[test]
fn tidy_learn_elapsed_threshold_ms_defaults_to_learn_min() {
    use crate::cli::LEARN_MIN_ELAPSED_MS;
    assert_eq!(
        crate::cli::tidy_flow::recovery::tidy_learn_elapsed_threshold_ms(),
        LEARN_MIN_ELAPSED_MS
    );
}

#[test]
fn tidy_fail_on_abort_ok_without_result_file() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "plan").expect("write plan");
    let artifacts =
        crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    crate::cli::tidy_flow::recovery::tidy_fail_on_abort(&artifacts).expect("no abort marker");
}

#[test]
fn merge_tidy_timing_returns_abort_when_result_file_present() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "plan").expect("write plan");
    let artifacts =
        crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    std::fs::write(artifacts.artifact_result_md(), "ABORT: tidy stop\n").expect("abort marker");
    let backups = crate::test_utils::empty_session_dotfile_backups(&artifacts.work_dir);
    let err = crate::cli::tidy_flow::merge_tidy_timing(Ok(()), &artifacts, &backups)
        .expect_err("abort");
    assert!(err.contains("ABORT: tidy stop"), "got {err:?}");
}

fn merge_tidy_timing_checks_fixture() -> (
    tempfile::TempDir,
    crate::artifacts::RunArtifacts,
    crate::artifacts::SessionDotfileBackups,
    std::path::PathBuf,
) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let checks = tmp.path().join(".malvin_checks");
    std::fs::write(&checks, "orig\n").expect("write checks");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "plan").expect("write plan");
    let artifacts =
        crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    let backups = crate::test_utils::empty_session_dotfile_backups(&artifacts.work_dir);
    (tmp, artifacts, backups, checks)
}

#[test]
fn merge_tidy_timing_restores_malvin_checks_after_workspace_mutation() {
    let (_tmp, artifacts, backups, checks) = merge_tidy_timing_checks_fixture();
    std::fs::write(&checks, "mutated\n").expect("mutate checks");
    crate::cli::tidy_flow::merge_tidy_timing(Ok(()), &artifacts, &backups).expect("merge");
    assert_eq!(
        std::fs::read_to_string(&checks).expect("read checks"),
        "orig\n"
    );
}

#[test]
fn tidy_prompt_context_loads_store_and_plan_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "tidy").expect("write plan");
    let artifacts =
        crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    let (_store, ctx) =
        crate::cli::tidy_flow::tidy_prompt_context(&artifacts).expect("tidy context");
    assert!(ctx.get("plan_path").is_some_and(|p| p.contains("plan.md")));
}

#[test]
fn tidy_recovery_request_and_paths_hold_attempt_metadata() {
    use crate::cli::tidy_flow::recovery::{
        TidyRecoveryPaths, TidyRecoveryRequest, TidyReviewAttemptOutcome,
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let backups = crate::test_utils::empty_session_dotfile_backups(tmp.path());
    let paths = TidyRecoveryPaths {
        work_dir: tmp.path().join("work"),
        run_dir: tmp.path().join("run"),
    };
    let req = TidyRecoveryRequest {
        attempt: 2,
        max_inner_retries: 3,
        session_dotfile_backups: &backups,
        paths: paths.clone(),
    };
    assert_eq!(req.attempt, 2);
    assert_eq!(req.max_inner_retries, 3);
    assert_eq!(paths.work_dir, tmp.path().join("work"));
    assert_eq!(TidyReviewAttemptOutcome::Lgtm, TidyReviewAttemptOutcome::Lgtm);
}
