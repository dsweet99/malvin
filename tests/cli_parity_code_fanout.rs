#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    acp_mock_code_fanout_reviewer_pollutes_workspace_js,
    acp_mock_code_fanout_skips_reviewer_outputs_js,
    acp_mock_code_fanout_workspace_only_lgtm_js,
    acp_mock_code_review_write_succeeds_on_second_review_attempt_js,
    acp_mock_code_review_write_workspace_only_lgtm_js, run_code_with_mock_js,
};
#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use std::process::Command;

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

#[test]
fn fanout_default_prompts_exist_on_head_commit() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    for rel in [
        "default_prompts/review_descriptions.md",
        "default_prompts/reviewer_template.md",
        "default_prompts/review_write.md",
    ] {
        let out = Command::new("git")
            .args(["cat-file", "-e", &format!("HEAD:{rel}")])
            .current_dir(manifest_dir)
            .output()
            .expect("git cat-file");
        assert!(
            out.status.success(),
            "fan-out prompt must exist on HEAD for clean clones (not only staged): {rel}\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn review_fix_rust_modules_are_tracked_in_git() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    for rel in [
        "src/orchestrator/workflow_merge.rs",
        "tests/common/acp_code_fanout_mocks.rs",
    ] {
        let on_disk = manifest_dir.join(rel);
        assert!(on_disk.is_file(), "missing required module on disk: {rel}");
        let out = Command::new("git")
            .args(["ls-files", "--error-unmatch", rel])
            .current_dir(manifest_dir)
            .output()
            .expect("git ls-files");
        assert!(
            out.status.success(),
            "module must be tracked in git (mod.rs already depends on it): {rel}\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn fanout_default_prompts_are_tracked_in_git() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    for rel in [
        "default_prompts/review_descriptions.md",
        "default_prompts/reviewer_template.md",
        "default_prompts/review_write.md",
    ] {
        let on_disk = manifest_dir.join(rel);
        assert!(on_disk.is_file(), "missing fan-out prompt on disk: {rel}");
        let out = Command::new("git")
            .args(["ls-files", "--error-unmatch", rel])
            .current_dir(manifest_dir)
            .output()
            .expect("git ls-files");
        assert!(
            out.status.success(),
            "fan-out prompt must be tracked in git for clean clones (include_str! in defaults.rs): {rel}\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
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
