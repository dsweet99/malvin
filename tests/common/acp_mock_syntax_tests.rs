use super::acp_code_fanout_mocks::acp_mock_code_review_write_succeeds_on_second_review_attempt_js;
use super::acp_core::acp_mock_code_with_run_dir_js;
use super::acp_do::{
    acp_mock_tidy_abort_after_first_coder_turn_js, acp_mock_tidy_fanout_lgtm_js,
    acp_mock_tidy_fanout_non_lgtm_js, acp_mock_tidy_fanout_non_lgtm_then_lgtm_js,
    acp_mock_tidy_review_write_succeeds_on_second_attempt_js,
};
use std::process::Command;

fn assert_mock_js_syntax_valid(js: &str) {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("mock.js");
    std::fs::write(&path, js).expect("write mock");
    let out = Command::new("node")
        .arg("--check")
        .arg(&path)
        .output()
        .expect("spawn node");
    assert!(
        out.status.success(),
        "mock JS syntax check failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn acp_mock_tidy_fanout_lgtm_js_passes_node_syntax_check() {
    assert_mock_js_syntax_valid(&acp_mock_tidy_fanout_lgtm_js());
}

#[test]
fn acp_mock_tidy_fanout_non_lgtm_js_passes_node_syntax_check() {
    assert_mock_js_syntax_valid(&acp_mock_tidy_fanout_non_lgtm_js());
}

#[test]
fn acp_mock_tidy_fanout_non_lgtm_then_lgtm_js_passes_node_syntax_check() {
    assert_mock_js_syntax_valid(&acp_mock_tidy_fanout_non_lgtm_then_lgtm_js());
}

#[test]
fn acp_mock_tidy_abort_after_first_coder_turn_js_passes_node_syntax_check() {
    assert_mock_js_syntax_valid(&acp_mock_tidy_abort_after_first_coder_turn_js());
}

#[test]
fn acp_mock_tidy_review_write_succeeds_on_second_attempt_js_passes_node_syntax_check() {
    assert_mock_js_syntax_valid(&acp_mock_tidy_review_write_succeeds_on_second_attempt_js());
}

#[test]
fn acp_mock_code_review_write_succeeds_on_second_review_attempt_js_passes_node_syntax_check() {
    assert_mock_js_syntax_valid(&acp_mock_code_review_write_succeeds_on_second_review_attempt_js());
}
