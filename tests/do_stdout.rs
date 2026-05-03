mod common;

#[cfg(unix)]
use common::{
    acp_mock_do_creates_grounding_and_kissconfig_js, acp_mock_do_streaming_update_js,
    acp_mock_do_tamper_grounding_and_kissconfig_js, acp_mock_do_tampers_grounding_js,
    assert_stdout_has_no_chrome, first_do_log_path, run_do_with_mock, run_do_with_named_mock_bin,
    run_malvin_do_home_workspace, stdout_lines_preserve_shape, test_home_workspace,
};

#[cfg_attr(unix, test)]
fn do_restores_workspace_grounding_after_mock_agent_overwrites() {
    let (out, _root, workspace) = run_do_with_named_mock_bin(
        "mock-agent-acp-do-grounding",
        &acp_mock_do_tampers_grounding_js(),
        &[],
        None,
    );
    assert!(
        out.status.success(),
        "malvin do failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let restored = std::fs::read_to_string(workspace.join("grounding.md")).expect("read grounding");
    assert_eq!(restored, "x");
}

#[cfg_attr(unix, test)]
fn do_restores_missing_grounding_and_kissconfig_when_agent_creates_them() {
    let (root, _home, workspace) = test_home_workspace();
    let _ = std::fs::remove_file(workspace.join("grounding.md"));
    let mock = root.path().join("mock-agent-acp-do-create-protected");
    common::write_mock_executable(&mock, &acp_mock_do_creates_grounding_and_kissconfig_js());
    let out = run_malvin_do_home_workspace(&workspace, &root.path().join("home"), &mock);
    assert!(
        out.status.success(),
        "malvin do failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!workspace.join("grounding.md").exists());
    assert!(!workspace.join(".kissconfig").exists());
}

#[cfg_attr(unix, test)]
fn do_restores_kissconfig_when_grounding_missing() {
    let (root, _home, workspace) = test_home_workspace();
    std::fs::write(workspace.join(".kissconfig"), "k\n").expect("write kissconfig");
    let _ = std::fs::remove_file(workspace.join("grounding.md"));
    let mock = root.path().join("mock-agent-acp-do-tamper-kiss");
    common::write_mock_executable(&mock, &acp_mock_do_tamper_grounding_and_kissconfig_js());
    let out = run_malvin_do_home_workspace(&workspace, &root.path().join("home"), &mock);
    assert!(
        out.status.success(),
        "malvin do failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let restored = std::fs::read_to_string(workspace.join(".kissconfig")).expect("read kissconfig");
    assert_eq!(restored, "k\n");
    assert_eq!(
        std::fs::read_to_string(workspace.join("grounding.md")).expect("read grounding"),
        "TAMPERED"
    );
}

#[cfg_attr(unix, test)]
fn do_stdout_shows_plain_output_without_jsonrpc_lines() {
    let out = run_do_with_mock(&[]);
    assert!(out.status.success(), "malvin do failed: {out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines = stdout_lines_preserve_shape(&out.stdout);
    assert!(
        lines.iter().any(|l| l == "agent message"),
        "expected raw agent line, got {lines:?}"
    );
    assert_stdout_has_no_chrome(&lines);
    assert!(!stdout.contains("hidden thought"), "stdout was {stdout:?}");
    assert!(!stdout.contains("\"jsonrpc\""), "stdout was {stdout:?}");
}

#[cfg_attr(unix, test)]
fn do_trace_log_contains_thought_when_hidden_from_stdout() {
    let (out, _root, workspace) = run_do_with_named_mock_bin(
        "mock-agent-acp-do",
        &acp_mock_do_streaming_update_js(),
        &[],
        None,
    );
    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("hidden thought"),
        "default do should hide thought on stdout: {stdout:?}"
    );
    let log_path = first_do_log_path(&workspace);
    let log = std::fs::read_to_string(&log_path).expect("do.log");
    assert!(
        log.contains("hidden thought"),
        "trace log should retain thought text; path={log_path:?}"
    );
}

#[cfg_attr(unix, test)]
fn do_repo_gates_keeps_gate_diagnostics_off_stdout() {
    let out = run_do_with_mock(&["--repo-gates"]);
    assert!(
        out.status.success(),
        "malvin do --repo-gates failed: {out:?}"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines = stdout_lines_preserve_shape(&out.stdout);
    assert!(lines.iter().any(|l| l == "agent message"), "got: {lines:?}");
    assert!(
        lines.iter().all(|l| !l.contains(":[malvin]:")),
        "did not expect tagged repo-gate stdout lines, got: {lines:?}"
    );
    assert!(!stdout.contains("\"jsonrpc\""), "stdout was {stdout:?}");
}
