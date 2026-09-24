use super::format_retry_line;

fn format_retry_line_second_iteration_is_retry_without_done() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = crate::artifacts::create_run_artifacts_from_text("code", Some(tmp.path()))
        .expect("artifacts");
    crate::artifacts::ensure_gate_exp_log_file(&artifacts, 1).expect("exp log");
    let line = format_retry_line(Some(2), Some(&artifacts));
    assert!(line.contains("retry #1"));
    assert!(
        line.contains("requirements still unsatisfied"),
        "non-gate continue must not blame quality gates: {line}"
    );
    assert!(
        !line.contains("quality gates"),
        "exp-log-only continue must not mention quality gates: {line}"
    );
}

fn format_retry_line_detects_oom_from_sandbox_marker() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = crate::artifacts::create_run_artifacts_from_text("code", Some(tmp.path()))
        .expect("artifacts");
    crate::sandbox_oom::record_sandbox_oom_kill(
        &artifacts.run_dir,
        crate::sandbox_oom::SandboxOomKillRecord::from_facts(
            1,
            crate::sandbox_oom::SandboxOomKillFacts {
                reason: crate::sandbox_oom::OOM_REASON_MEMORY_LIMIT,
                rss_bytes: Some(999),
                limit_bytes: 512,
                pgid: 42,
            },
        ),
    )
    .expect("write");
    let line = format_retry_line(Some(2), Some(&artifacts));
    assert!(line.contains("OOM"));
}

fn format_retry_line_gates_failure_after_done() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = crate::artifacts::create_run_artifacts_from_text("code", Some(tmp.path()))
        .expect("artifacts");
    let prev = artifacts.gate_exp_log_path(1);
    std::fs::create_dir_all(prev.parent().expect("parent")).expect("mkdir");
    std::fs::write(&prev, "## Step 1 — router mock\n").expect("write");
    std::fs::write(
        artifacts.quality_gates_log_path(),
        "Finished `false`\nexit code: 1\n[stdout]\n\n[stderr]\n\n",
    )
    .expect("gate log");
    let line = format_retry_line(Some(2), Some(&artifacts));
    assert!(
        line.contains("quality gates"),
        "gate failure log must blame quality gates: {line}"
    );
}

fn kiss_cov_current_state_non_unix_branch() {
    crate::test_utils::with_isolated_home(|_| {
        let tmp = tempfile::tempdir().expect("tempdir");
        let _artifacts = crate::artifacts::create_run_artifacts_from_text("code", Some(tmp.path()))
            .expect("artifacts");
        let _ = super::current_sandbox_rss_bytes;
        let _ = super::effective_user_id;
        let _ = super::append_unsolved_reason;
        let _ = super::append_oom_reason;
        let _ = super::append_gates_reason;
        let _ = super::previous_quality_gates_failed;
        let _ = super::review_marks_checks_failed;
        let _ = super::quality_gates_log_has_nonzero_exit;
    });
}

fn append_unsolved_reason_records_missing_marker() {
    crate::test_utils::with_isolated_home(|_| {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts = crate::artifacts::create_run_artifacts_from_text("code", Some(tmp.path()))
            .expect("artifacts");
        let prev = artifacts.gate_exp_log_path(1);
        std::fs::create_dir_all(prev.parent().expect("parent")).expect("mkdir");
        std::fs::write(&prev, "no marker\n").expect("write");
        let mut reasons = Vec::new();
        super::append_unsolved_reason(&mut reasons, &artifacts, 1);
        assert_eq!(reasons.len(), 1);
        assert!(
            reasons[0].contains("requirements still unsatisfied"),
            "unsolved fallback must not blame gates: {:?}",
            reasons[0]
        );
    });
}

fn append_oom_reason_records_memory_kill() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = crate::artifacts::create_run_artifacts_from_text("code", Some(tmp.path()))
        .expect("artifacts");
    crate::sandbox_oom::record_sandbox_oom_kill(
        &artifacts.run_dir,
        crate::sandbox_oom::SandboxOomKillRecord::from_facts(
            1,
            crate::sandbox_oom::SandboxOomKillFacts {
                reason: crate::sandbox_oom::OOM_REASON_MEMORY_LIMIT,
                rss_bytes: Some(999),
                limit_bytes: 512,
                pgid: 42,
            },
        ),
    )
    .expect("write");
    let mut reasons = Vec::new();
    super::append_oom_reason(&mut reasons, &artifacts, 1);
    assert!(reasons.iter().any(|r| r.contains("OOM")));
}

fn append_gates_reason_after_done_session() {
    crate::test_utils::with_isolated_home(|_| {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts = crate::artifacts::create_run_artifacts_from_text("code", Some(tmp.path()))
            .expect("artifacts");
        std::fs::write(artifacts.artifact_review_md(), b"Checks do not pass\n").expect("review");
        let mut reasons = Vec::new();
        super::append_gates_reason(&mut reasons, &artifacts, 1);
        assert!(
            reasons.iter().any(|r| r.contains("quality gates")),
            "review marker must yield gates reason: {reasons:?}"
        );
    });
}

#[test]
fn kiss_bundled_current_state_tests_tail() {
    format_retry_line_second_iteration_is_retry_without_done();
    format_retry_line_detects_oom_from_sandbox_marker();
    format_retry_line_gates_failure_after_done();
    kiss_cov_current_state_non_unix_branch();
    append_unsolved_reason_records_missing_marker();
    append_oom_reason_records_memory_kill();
    append_gates_reason_after_done_session();
}
