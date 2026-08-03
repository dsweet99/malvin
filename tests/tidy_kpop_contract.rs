//! `malvin tidy` is a router-backed request wrapper with `--gates` forced on.

#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    TidySpawn, acp_mock_router_no_work_js, bin_path_with_failing_gates, bin_path_with_fake_kiss,
    combined_cli_output, fast_test_home_workspace, seed_malvin_checks, spawn_tidy,
    cached_mock_executable,
};

#[cfg(unix)]
#[test]
fn tidy_router_succeeds_when_gates_pass() {
    let (root, home, workspace) = fast_test_home_workspace();
    seed_malvin_checks(&workspace, "true\n");
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable(&acp_mock_router_no_work_js());
    let out = spawn_tidy(&TidySpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "1"],
        gate_trace: None,
    });
    let combined = combined_cli_output(&out);
    assert!(
        out.status.success(),
        "tidy must succeed via router when gates pass: {combined:?}"
    );
    assert!(
        combined.contains("Get the gates to pass."),
        "startup must emit the fixed tidy request: {combined:?}"
    );
    assert!(
        combined.contains("router_a") || combined.contains("__MALVIN_DONE__") || combined.contains("router_header"),
        "tidy must run the default router workflow: {combined:?}"
    );
}

#[cfg(unix)]
#[test]
fn tidy_forces_gates_even_without_cli_gates_flag() {
    let (root, home, workspace) = fast_test_home_workspace();
    seed_malvin_checks(&workspace, "lint\n");
    let trace = root.path().join("gate-trace.log");
    let path = bin_path_with_failing_gates(&root, &trace);
    let mock = cached_mock_executable(&acp_mock_router_no_work_js());
    let out = spawn_tidy(&TidySpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        // No `--gates` in extra_args: tidy must still run harness gates.
        extra_args: &["--max-loops", "1"],
        gate_trace: Some(&trace),
    });
    let combined = combined_cli_output(&out);
    assert!(
        !out.status.success(),
        "expected tidy to fail when harness gates fail: {combined:?}"
    );
    let trace_log = std::fs::read_to_string(&trace).unwrap_or_default();
    assert!(
        trace_log.contains("lint"),
        "expected harness quality gate run without CLI --gates: {trace_log}"
    );
}

#[cfg(unix)]
#[test]
fn tidy_fails_when_post_session_gates_still_fail() {
    let (root, home, workspace) = fast_test_home_workspace();
    seed_malvin_checks(&workspace, "lint\n");
    let trace = root.path().join("gate-trace.log");
    let path = bin_path_with_failing_gates(&root, &trace);
    let mock = cached_mock_executable(&acp_mock_router_no_work_js());
    let out = spawn_tidy(&TidySpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "0"],
        gate_trace: Some(&trace),
    });
    assert!(
        !out.status.success(),
        "expected tidy to fail when post-router gates fail: {out:?}"
    );
    let trace_log = std::fs::read_to_string(&trace).unwrap_or_default();
    assert!(
        trace_log.contains("lint"),
        "expected post-router quality gate run: {trace_log}"
    );
}
