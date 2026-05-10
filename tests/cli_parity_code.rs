mod common;

#[cfg(unix)]
use common::{
    CodeRunOpts, MAX_LOOPS_EXHAUSTED, acp_mock_code_abort_after_implement_js,
    acp_mock_code_abort_result_after_check_plan_lgtm_js,
    acp_mock_code_check_plan_tampers_kissconfig_then_implement_verifies_restore_js,
    acp_mock_code_review_lgtm_to_artifact_js, acp_mock_code_review_lgtm_with_abort_js,
    assert_review_abort_behavior, only_run_dir, run_code_max_loops_zero_with_mock,
    run_code_max_loops_zero_with_mock_without_trust_plan, run_code_with_mock_js,
    run_code_with_mock_js_trust_plan, run_code_with_mock_js_trust_plan_in_workspace,
};

#[cfg_attr(unix, test)]
fn code_stops_when_implement_writes_abort_result() {
    let out = run_code_with_mock_js(
        &acp_mock_code_abort_after_implement_js(),
        &["--max-loops", "1"],
        true,
    );
    assert!(
        !out.status.success(),
        "expected ABORT failure path: {out:?}"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("ABORT: stop now"),
        "expected implement ABORT to stop the workflow: {combined:?}"
    );
    assert!(
        !combined.contains(MAX_LOOPS_EXHAUSTED),
        "workflow should stop on ABORT before review exhaustion: {combined:?}"
    );
}

#[cfg_attr(unix, test)]
fn code_stops_when_check_plan_writes_abort_result_with_lgtm_review() {
    let out = run_code_with_mock_js_trust_plan(
        &acp_mock_code_abort_result_after_check_plan_lgtm_js(),
        &["--max-loops", "1"],
        &CodeRunOpts {
            no_tee: true,
            trust_plan: false,
        },
    );
    assert!(
        !out.status.success(),
        "expected ABORT failure path: {out:?}"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("ABORT: after check plan"),
        "expected check_plan ABORT to stop the workflow: {combined:?}"
    );
    assert!(
        !combined.contains("implement_phase_ran"),
        "implement must not run after ABORT in result.md from check_plan: {combined:?}"
    );
}

#[cfg_attr(unix, test)]
fn check_plan_kissconfig_restore_happens_before_implement() {
    let out = run_code_with_mock_js_trust_plan(
        &acp_mock_code_check_plan_tampers_kissconfig_then_implement_verifies_restore_js(),
        &["--max-loops", "1"],
        &CodeRunOpts {
            no_tee: false,
            trust_plan: false,
        },
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("implement ok"),
        "expected implement to see restored kissconfig: {combined:?}"
    );
    assert!(
        !combined.contains("ABORT: kissconfig leaked into implement"),
        "check_plan kissconfig mutation must not leak into implement: {combined:?}"
    );
    assert!(
        out.status.success(),
        "expected successful exit when check_plan + implement restore path converges: {combined:?}"
    );
}

#[cfg_attr(unix, test)]
fn max_loops_zero_skips_review_attempts_and_fails() {
    let out = run_code_max_loops_zero_with_mock();
    assert!(
        !out.status.success(),
        "malvin code unexpectedly succeeded: {out:?}"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains(MAX_LOOPS_EXHAUSTED),
        "expected max_loops=0 review skip failure: {combined:?}"
    );
    assert!(
        combined.contains("Review-1 (attempt 1)"),
        "review-1 should run at least once when --max-loops=0: {combined:?}"
    );
}

#[cfg_attr(unix, test)]
fn max_loops_zero_skips_check_plan_attempt() {
    let out = run_code_max_loops_zero_with_mock_without_trust_plan();
    assert!(
        !out.status.success(),
        "malvin code unexpectedly succeeded: {out:?}"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("CheckPlan"),
        "check_plan should run at least once when max_loops=0: {combined:?}"
    );
    assert!(
        !combined.contains("Review-1 (attempt 1)"),
        "review attempt must not run when --max-loops=0: {combined:?}"
    );
    assert!(
        combined.contains("check_plan: agent did not write review file after retries"),
        "expected check_plan missing-review failure path: {combined:?}"
    );
}

#[cfg_attr(unix, test)]
fn review_loop_accepts_lgtm_written_to_artifact_path() {
    let out = run_code_with_mock_js(
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
        "review loop should accept LGTM from artifact path: {combined:?}"
    );
    assert!(
        out.status.success(),
        "malvin code should succeed when reviewer writes LGTM to artifact: {combined:?}"
    );
}

#[cfg_attr(unix, test)]
fn code_stops_when_review_lgtm_also_writes_abort_result() {
    let out = run_code_with_mock_js(
        &acp_mock_code_review_lgtm_with_abort_js(),
        &["--max-loops", "1"],
        true,
    );
    assert_review_abort_behavior(
        &out,
        "ABORT: review lgtm abort test",
        "Review-2 (attempt 1)",
    );
}

#[cfg_attr(unix, test)]
fn skip_pre_checks_skips_initial_repo_gates_in_quality_log() {
    let js = acp_mock_code_review_lgtm_to_artifact_js();
    let opts = CodeRunOpts {
        no_tee: true,
        trust_plan: true,
    };
    let (out, _root, workspace) = run_code_with_mock_js_trust_plan_in_workspace(
        &js,
        &["--max-loops", "1", "--skip-pre-checks"],
        &opts,
    );
    assert!(
        out.status.success(),
        "malvin code should succeed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let log = std::fs::read_to_string(only_run_dir(&workspace).join("quality_checks.log"))
        .expect("quality_checks.log");
    assert_eq!(
        log.matches("Running `kiss check`").count(),
        1,
        "expected one gate pass (pre-summary only): {log}"
    );

    let (out2, _root2, workspace2) =
        run_code_with_mock_js_trust_plan_in_workspace(&js, &["--max-loops", "1"], &opts);
    assert!(out2.status.success(), "baseline malvin code should succeed");
    let log2 = std::fs::read_to_string(only_run_dir(&workspace2).join("quality_checks.log"))
        .expect("quality_checks.log baseline");
    assert_eq!(
        log2.matches("Running `kiss check`").count(),
        2,
        "expected initial plus pre-summary gate passes: {log2}"
    );
}
