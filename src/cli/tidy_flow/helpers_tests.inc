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
    use malvin::review_sync::{is_lgtm_str, sync_review_file_for_attempt};

    let t = tempfile::tempdir().expect("tempdir");
    let artifacts =
        malvin::artifacts::create_run_artifacts_from_text("gate_marker", Some(t.path()))
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
fn compose_tidy_concerns_includes_review_path_when_present_in_context() {
    let store = malvin::prompts::PromptStore::default_store();
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
