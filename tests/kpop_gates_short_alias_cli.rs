//! Bare `malvin -g` vs `--gates` harness parity.
#![cfg(unix)]

mod common;

use common::{
    acp_mock_router_no_work_js, bin_path_with_failing_gates, cached_mock_executable,
    combined_cli_output, fast_test_home_workspace, seed_malvin_checks, INTEGRATION_TEST_MALVIN_ARGS,
    MALVIN_TEST_CMD_TIMEOUT, command_output_with_timeout,
};
use std::path::Path;
use std::process::Command;

struct BareGatesSpawn<'a> {
    workspace: &'a Path,
    home: &'a Path,
    mock: &'a Path,
    path_var: &'a str,
    gates_flag: &'a str,
    gate_trace: &'a Path,
}

fn spawn_bare_gates(t: &BareGatesSpawn<'_>) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.current_dir(t.workspace)
        .env("HOME", t.home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", t.mock)
        .env("PATH", t.path_var)
        .env("MALVIN_TEST_GATE_TRACE", t.gate_trace);
    let mut args: Vec<&str> = vec![t.gates_flag, "noop request for kpop"];
    args.extend_from_slice(INTEGRATION_TEST_MALVIN_ARGS);
    args.extend_from_slice(&["--max-loops", "1"]);
    cmd.args(args);
    command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT).expect("spawn malvin")
}

#[test]
fn bare_short_g_matches_long_gates_on_failing_checks() {
    let (root, home, workspace) = fast_test_home_workspace();
    seed_malvin_checks(&workspace, "lint\n");
    let mock = cached_mock_executable(&acp_mock_router_no_work_js());

    let trace_long = root.path().join("gate-trace-long.log");
    let path_long = bin_path_with_failing_gates(&root, &trace_long);
    let out_long = spawn_bare_gates(&BareGatesSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path_long,
        gates_flag: "--gates",
        gate_trace: &trace_long,
    });

    let trace_short = root.path().join("gate-trace-short.log");
    let path_short = bin_path_with_failing_gates(&root, &trace_short);
    let out_short = spawn_bare_gates(&BareGatesSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path_short,
        gates_flag: "-g",
        gate_trace: &trace_short,
    });

    let long_combined = combined_cli_output(&out_long);
    let short_combined = combined_cli_output(&out_short);
    assert_eq!(
        out_long.status.success(),
        out_short.status.success(),
        "success parity long={long_combined:?} short={short_combined:?}"
    );
    assert!(
        !out_short.status.success(),
        "expected -g harness to fail on lint: {short_combined:?}"
    );
    let long_trace = std::fs::read_to_string(&trace_long).unwrap_or_default();
    let short_trace = std::fs::read_to_string(&trace_short).unwrap_or_default();
    assert!(
        long_trace.contains("lint"),
        "expected --gates to run lint: {long_trace}"
    );
    assert!(
        short_trace.contains("lint"),
        "expected -g to run lint: {short_trace}"
    );
}
