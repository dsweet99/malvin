//! Tidy quality-gate and ACP ordering checks (router-backed, gates forced on).

#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    MALVIN_TEST_CMD_TIMEOUT, INTEGRATION_TEST_MALVIN_ARGS,
    acp_mock_router_no_work_js, command_output_with_timeout, seed_malvin_checks,
    write_failing_gate_tools, cached_mock_executable, fast_test_home_workspace,
    bin_path_with_fake_kiss, combined_cli_output,
};
#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use std::process::Command;

#[cfg(unix)]
struct MalvinTidySpawn<'a> {
    workspace: &'a Path,
    home: &'a Path,
    mock: &'a Path,
    path: &'a str,
    trace: Option<&'a Path>,
    timeout: std::time::Duration,
}

#[cfg(unix)]
fn spawn_malvin_tidy(c: &MalvinTidySpawn<'_>) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.current_dir(c.workspace)
        .env("HOME", c.home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", c.mock)
        .env("PATH", c.path)
        .args(["tidy"]);
    if let Some(trace) = c.trace {
        cmd.env("MALVIN_TEST_GATE_TRACE", trace);
    }
    cmd.args(INTEGRATION_TEST_MALVIN_ARGS);
    cmd.args(["--max-loops", "0"]);
    command_output_with_timeout(&mut cmd, c.timeout).expect("spawn malvin")
}

#[cfg_attr(unix, test)]
fn malvin_tidy_runs_router_when_gates_already_pass() {
    let (root, home, workspace) = fast_test_home_workspace();
    seed_malvin_checks(&workspace, "true\n");
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable(&acp_mock_router_no_work_js());
    let out = spawn_malvin_tidy(&MalvinTidySpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path: &path,
        trace: None,
        timeout: MALVIN_TEST_CMD_TIMEOUT,
    });
    let combined = combined_cli_output(&out);
    assert!(
        out.status.success(),
        "expected tidy router success when gates pass; {combined:?}"
    );
    assert!(
        combined.contains("Get the gates to pass."),
        "expected fixed tidy request in output; {combined:?}"
    );
    assert!(
        combined.contains("router_requirements") || combined.contains("NO_WORK_REMAINING"),
        "agent must run via default router even when gates already pass; {combined:?}"
    );
}

#[cfg_attr(unix, test)]
fn malvin_tidy_runs_quality_gates_after_router_when_gates_fail() {
    let (root, home, workspace) = fast_test_home_workspace();
    seed_malvin_checks(&workspace, "lint\n");
    let bin_dir = root.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    let trace = root.path().join("quality-trace.log");
    write_failing_gate_tools(&bin_dir, &trace);
    let mock = cached_mock_executable(&acp_mock_router_no_work_js());
    let original_path = std::env::var("PATH").unwrap_or_default();
    let path = format!("{}:{original_path}", bin_dir.display());

    let out = spawn_malvin_tidy(&MalvinTidySpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path: &path,
        trace: Some(&trace),
        timeout: MALVIN_TEST_CMD_TIMEOUT,
    });

    assert!(
        !out.status.success(),
        "expected tidy to fail when post-router quality gates fail: {out:?}"
    );
    let trace_log = std::fs::read_to_string(&trace).unwrap_or_default();
    assert!(
        !trace_log.is_empty(),
        "expected quality gates to run after router: {trace_log}"
    );
    assert!(
        trace_log.contains("lint"),
        "expected at least one quality gate command in trace: {trace_log}"
    );
}
