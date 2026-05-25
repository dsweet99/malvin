#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    acp_mock_code_fanout_reviewer_pollutes_workspace_js,
    acp_mock_code_fanout_skips_reviewer_outputs_js, acp_mock_code_fanout_workspace_only_lgtm_js,
    acp_mock_code_missing_artifact_recovers_on_outer_review_attempt_js,
    acp_mock_code_review_write_never_writes_artifact_js,
    acp_mock_code_review_write_succeeds_on_second_review_attempt_js,
    acp_mock_code_review_write_workspace_only_lgtm_js, run_code_with_mock_js,
};
#[cfg(unix)]
use std::path::Path;

#[cfg_attr(unix, test)]
fn code_fails_when_review_omits_prep() {
    let out = run_code_with_mock_js(
        &acp_mock_code_fanout_skips_reviewer_outputs_js(),
        &["--max-loops", "1", "--skip-pre-checks"],
        true,
    );
    assert!(!out.status.success(), "expected code failure: {out:?}");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("review did not write review prep"),
        "expected review prep guard before review_write: {combined:?}"
    );
}

#[cfg_attr(unix, test)]
fn review_loop_rejects_workspace_lgtm_when_review_write_omits_artifact() {
    let out = run_code_with_mock_js(
        &acp_mock_code_review_write_workspace_only_lgtm_js(),
        &["--max-loops", "1"],
        true,
    );
    assert!(
        !out.status.success(),
        "malvin code must not succeed on workspace-only LGTM: {out:?}"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("review_write did not write artifact review"),
        "expected explicit review_write artifact guard: {combined:?}"
    );
}

#[test]
fn review_default_prompts_exist_on_disk() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    for rel in [
        "default_prompts/review.md",
        "default_prompts/review_write.md",
    ] {
        assert!(
            manifest_dir.join(rel).is_file(),
            "missing review prompt: {rel}"
        );
    }
}

#[test]
fn review_fix_rust_modules_exist_on_disk() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    for rel in [
        "src/orchestrator/workflow_merge.rs",
        "src/orchestrator/review_write_retry.rs",
        "tests/common/acp_code_fanout_mocks.rs",
    ] {
        let on_disk = manifest_dir.join(rel);
        assert!(on_disk.is_file(), "missing required module on disk: {rel}");
    }
}

#[test]
fn review_default_prompts_are_embedded_files() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    for rel in [
        "default_prompts/review.md",
        "default_prompts/review_write.md",
    ] {
        let on_disk = manifest_dir.join(rel);
        assert!(on_disk.is_file(), "missing embedded prompt on disk: {rel}");
    }
}

#[test]
fn review_tidy_contract_tests_exist_on_disk() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let rel = "tests/tidy_kpop_contract.rs";
    assert!(
        manifest_dir.join(rel).is_file(),
        "missing tidy kpop contract test on disk: {rel}"
    );
}

#[cfg_attr(unix, test)]
fn code_missing_artifact_recovers_on_outer_review_attempt_with_new_fanout() {
    let out = run_code_with_mock_js(
        &acp_mock_code_missing_artifact_recovers_on_outer_review_attempt_js(),
        &["--max-loops", "2", "--skip-pre-checks"],
        true,
    );
    assert!(
        out.status.success(),
        "expected outer review attempt to re-fan-out after inner review_write retries exhaust: {out:?}"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !combined.contains("review: review_write did not write artifact review after retries"),
        "inner exhaustion should not fail the phase when another outer attempt can re-fan-out: {combined:?}"
    );
}

#[cfg_attr(unix, test)]
fn review_write_missing_artifact_retries_within_max_loops() {
    let out = run_code_with_mock_js(
        &acp_mock_code_review_write_succeeds_on_second_review_attempt_js(),
        &["--max-loops", "2", "--skip-pre-checks"],
        true,
    );
    assert!(
        out.status.success(),
        "expected second review attempt after review_write omits artifact on first try: {out:?}"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !combined.contains("review: review_write did not write artifact review after retries"),
        "retry should recover from missing artifact on first review_write, got: {combined:?}"
    );
}

#[cfg_attr(unix, test)]
fn review_write_missing_artifact_exhaustion_errors_after_max_loops() {
    let out = run_code_with_mock_js(
        &acp_mock_code_review_write_never_writes_artifact_js(),
        &["--max-loops", "1", "--skip-pre-checks"],
        true,
    );
    assert!(
        !out.status.success(),
        "expected failure when artifact never written: {out:?}"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("review: review_write did not write artifact review after retries"),
        "expected exhaustion error when review_write never writes artifact: {combined:?}"
    );
}

#[cfg_attr(unix, test)]
fn fanout_workspace_lgtm_pollution_does_not_false_lgtm_when_artifact_has_problems() {
    let out = run_code_with_mock_js(
        &acp_mock_code_fanout_reviewer_pollutes_workspace_js(),
        &["--max-loops", "1", "--skip-pre-checks"],
        true,
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !combined.contains("review_write did not write artifact review"),
        "review_write must still write artifact when reviewers pollute workspace: {combined:?}"
    );
    assert!(
        combined.contains("Concerns (attempt 1)"),
        "non-LGTM artifact must run concerns, not exit review as LGTM: {combined:?}"
    );
}

#[cfg_attr(unix, test)]
fn workspace_lgtm_during_fanout_without_artifact_fails_explicitly() {
    let out = run_code_with_mock_js(
        &acp_mock_code_fanout_workspace_only_lgtm_js(),
        &["--max-loops", "1", "--skip-pre-checks"],
        true,
    );
    assert!(
        !out.status.success(),
        "workspace-only LGTM during fan-out must not complete malvin code: {out:?}"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("review_write did not write artifact review"),
        "expected artifact guard when reviewers leave workspace LGTM only: {combined:?}"
    );
}
