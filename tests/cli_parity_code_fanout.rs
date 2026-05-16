#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{acp_mock_code_fanout_skips_reviewer_outputs_js, run_code_with_mock_js};

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
