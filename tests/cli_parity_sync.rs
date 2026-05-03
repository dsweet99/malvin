mod common;

#[cfg(unix)]
use common::{
    acp_mock_code_check_sync_then_review_lgtm_js, acp_mock_code_review_lgtm_to_artifact_js,
    acp_mock_sync_check_sync_non_exact_lgtm_js, acp_mock_sync_header_capture_js,
    acp_mock_sync_review_lgtm_with_abort_js, acp_mock_sync_reviewer_restore_between_attempts_js,
    acp_mock_sync_tamper_and_review_restore_js, assert_review_abort_behavior,
    assert_sync_tamper_flow_restores_grounding_and_fails, only_run_dir, run_sync_with_mock_js,
    run_sync_with_mock_js_and_workspace, run_sync_with_mock_js_max_loops_zero, CHECK_SYNC_PROMPT,
    MAX_LOOPS_EXHAUSTED, SyncRunOpts,
};

#[cfg_attr(unix, test)]
fn sync_stops_when_review_lgtm_also_writes_abort_result() {
    let out = run_sync_with_mock_js(
        &acp_mock_sync_review_lgtm_with_abort_js(),
        &["--max-loops", "2"],
        true,
    );
    assert_review_abort_behavior(
        &out,
        "ABORT: sync review LGTM abort test",
        "Review-2 (attempt 1)",
    );
}

#[cfg_attr(unix, test)]
fn sync_accepts_review_lgtm_written_to_artifact_path() {
    let out = run_sync_with_mock_js(
        &acp_mock_code_review_lgtm_to_artifact_js(),
        &["--max-loops", "1"],
        true,
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !combined.contains(MAX_LOOPS_EXHAUSTED),
        "sync should succeed with LGTM from artifact: {combined:?}"
    );
    assert!(
        out.status.success(),
        "malvin sync should succeed when review writes LGTM: {combined:?}"
    );
}

#[cfg_attr(unix, test)]
fn sync_max_loops_zero_skips_review_attempts_and_fails() {
    let out = run_sync_with_mock_js_max_loops_zero();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !out.status.success(),
        "sync should fail without reviews: {combined:?}"
    );
    assert!(
        combined.contains("Did not receive LGTM for check_sync.md within max loops.")
            || combined.contains(MAX_LOOPS_EXHAUSTED),
        "expected max_loops skip failure: {combined:?}"
    );
    assert!(
        !combined.contains("Review-1 (attempt 1)"),
        "review attempt must not run when --max-loops=0: {combined:?}"
    );
}

#[cfg_attr(unix, test)]
fn sync_runs_check_sync_before_review_1() {
    let (out, _root, workspace) = run_sync_with_mock_js_and_workspace(
        &acp_mock_code_check_sync_then_review_lgtm_js(),
        &["--max-loops", "2"],
        &SyncRunOpts {
            no_tee: true,
            with_kissconfig: false,
        },
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.status.success(),
        "sync should succeed when check_sync and review_1 both hit LGTM: {combined:?}"
    );

    let check_sync_index = combined
        .find("CheckSync (attempt 1)")
        .expect("check_sync progress line");
    let review_index = combined
        .find("Review-1 (attempt 1)")
        .expect("review-1 progress line");
    assert!(
        check_sync_index < review_index,
        "expected check_sync to run before review_1: {combined:?}"
    );
    let run_dir = only_run_dir(&workspace);
    let has_check_sync_log = std::fs::read_dir(&run_dir)
        .expect("run dir")
        .filter_map(Result::ok)
        .any(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .contains("coder_check_sync")
        });
    assert!(
        has_check_sync_log,
        "expected check_sync coder log to capture session/prompt request: {combined:?}"
    );
}

#[cfg_attr(unix, test)]
fn sync_rejects_non_exact_lgtm_from_check_sync() {
    let out = run_sync_with_mock_js(
        &acp_mock_sync_check_sync_non_exact_lgtm_js(),
        &["--max-loops", "2"],
        true,
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !out.status.success(),
        "non-exact check_sync LGTM text should fail: {combined:?}"
    );
    assert!(
        combined.contains("Did not receive LGTM for check_sync.md within max loops."),
        "expected strict check_sync parsing behavior: {combined:?}"
    );
    assert!(
        !combined.contains("Review-1 (attempt 1)"),
        "check_sync should fail before review_1 on invalid LGTM: {combined:?}"
    );
}

#[cfg_attr(unix, test)]
fn check_sync_prompt_requires_exact_lgtm_instruction() {
    assert!(
        CHECK_SYNC_PROMPT.contains("write *only* and *exactly* LGTM"),
        "check_sync prompt should require exact LGTM wording"
    );
}

#[cfg_attr(unix, test)]
fn sync_prepends_header_to_review_prompts() {
    let (out, _root, workspace) = run_sync_with_mock_js_and_workspace(
        &acp_mock_sync_header_capture_js(),
        &["--max-loops", "2"],
        &SyncRunOpts {
            no_tee: true,
            with_kissconfig: false,
        },
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.status.success(),
        "sync should succeed when sync prompts all include header and reviews reach LGTM: {combined:?}"
    );
    let run_dir = only_run_dir(&workspace);
    let marker = std::fs::read_to_string(run_dir.join("sync_prompt_headers.txt"))
        .expect("read sync_prompt_headers marker");
    let lines: Vec<&str> = marker.lines().collect();
    assert!(
        !lines.is_empty(),
        "expected header markers to be emitted: {marker:?}"
    );
    assert!(
        lines.iter().all(|line| *line == "header"),
        "expected header prepended to every sync prompt: {marker:?}"
    );
    assert!(
        !marker.contains("missing"),
        "expected no missing-header prompts: {marker:?}"
    );
}

#[cfg_attr(unix, test)]
fn sync_check_sync_tamper_and_restore_before_review_1() {
    let (out, _root, workspace) = run_sync_with_mock_js_and_workspace(
        &acp_mock_sync_tamper_and_review_restore_js(),
        &["--max-loops", "2"],
        &SyncRunOpts {
            no_tee: true,
            with_kissconfig: true,
        },
    );
    assert_sync_tamper_flow_restores_grounding_and_fails(&out, &workspace);
}

#[cfg_attr(unix, test)]
fn sync_reviewer_restores_between_reviewer_attempts() {
    let (out, _root, workspace) = run_sync_with_mock_js_and_workspace(
        &acp_mock_sync_reviewer_restore_between_attempts_js(),
        &["--max-loops", "3"],
        &SyncRunOpts {
            no_tee: true,
            with_kissconfig: true,
        },
    );
    assert_sync_tamper_flow_restores_grounding_and_fails(&out, &workspace);
}
