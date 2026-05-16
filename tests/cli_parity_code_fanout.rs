#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    acp_mock_code_fanout_skips_reviewer_outputs_js, acp_mock_code_review_write_workspace_only_lgtm_js,
    run_code_with_mock_js,
};

#[cfg_attr(unix, test)]
fn code_fails_when_fanout_mock_skips_reviewer_outputs() {
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
        combined.contains("missing reviewer output"),
        "expected fan-out preflight failure before review_write: {combined:?}"
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

#[cfg_attr(unix, test)]
fn review_write_omitting_artifact_surfaces_explicit_error() {
    let out = run_code_with_mock_js(
        &acp_mock_code_review_write_workspace_only_lgtm_js(),
        &["--max-loops", "1"],
        true,
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("review_write did not write artifact review"),
        "expected check_plan-style artifact guard after review_write, got: {combined:?}"
    );
}
