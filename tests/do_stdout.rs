mod common;

#[cfg(unix)]
use common::{
    acp_mock_do_creates_kissconfig_js, acp_mock_do_creates_kissignore_js,
    acp_mock_do_streaming_update_js,
    acp_mock_do_tampers_kissconfig_js, acp_mock_do_tampers_kissconfig_js_only,
    acp_mock_do_tampers_kissignore_js, acp_mock_do_tampers_kissignore_js_only,
    acp_mock_do_tampers_malvin_checks_js, acp_mock_do_tampers_malvin_checks_js_only,
    assert_stdout_has_no_chrome, first_do_log_path,
    run_do_with_mock, run_do_with_mock_force_tee, run_do_with_named_mock_bin, run_malvin_do_home_workspace,
    stdout_lines_preserve_shape, test_home_workspace,
};

#[cfg_attr(unix, test)]
fn do_restores_workspace_kissconfig_after_mock_agent_overwrites() {
    let (out, _root, workspace) = run_do_with_named_mock_bin(
        "mock-agent-acp-do-kissconfig",
        &acp_mock_do_tampers_kissconfig_js(),
        &[],
        None,
    );
    assert!(
        out.status.success(),
        "malvin do failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let restored = std::fs::read_to_string(workspace.join(".kissconfig")).expect("read kissconfig");
    assert_eq!(restored, "x");
}

#[cfg_attr(unix, test)]
fn do_restores_missing_kissconfig_when_agent_creates_it() {
    let (root, _home, workspace) = test_home_workspace();
    let _ = std::fs::remove_file(workspace.join(".kissconfig"));
    let mock = root.path().join("mock-agent-acp-do-create-protected");
    common::write_mock_executable(&mock, &acp_mock_do_creates_kissconfig_js());
    let out = run_malvin_do_home_workspace(&workspace, &root.path().join("home"), &mock);
    assert!(
        out.status.success(),
        "malvin do failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!workspace.join(".kissconfig").exists());
}

#[cfg_attr(unix, test)]
fn do_restores_workspace_malvin_checks_after_mock_agent_overwrites() {
    let (root, home, workspace) = test_home_workspace();
    common::seed_malvin_checks(&workspace, "x\n");
    let mock = root.path().join("mock-agent-acp-do-malvin-checks");
    common::write_mock_executable(&mock, &acp_mock_do_tampers_malvin_checks_js());
    let out = run_malvin_do_home_workspace(&workspace, &home, &mock);
    assert!(
        out.status.success(),
        "malvin do failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let restored =
        std::fs::read_to_string(workspace.join(".malvin/checks")).expect("read .malvin/checks");
    assert_eq!(restored, "x\n");
}

#[cfg_attr(unix, test)]
fn do_restores_malvin_checks_after_tamper_when_present_at_start() {
    let (root, _home, workspace) = test_home_workspace();
    common::seed_malvin_checks(&workspace, "m\n");
    let mock = root.path().join("mock-agent-acp-do-tamper-malvin");
    common::write_mock_executable(&mock, &acp_mock_do_tampers_malvin_checks_js_only());
    let out = run_malvin_do_home_workspace(&workspace, &root.path().join("home"), &mock);
    assert!(
        out.status.success(),
        "malvin do failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let restored =
        std::fs::read_to_string(workspace.join(".malvin/checks")).expect("read .malvin/checks");
    assert_eq!(restored, "m\n");
}

#[cfg_attr(unix, test)]
fn do_restores_kissconfig_after_tamper_when_present_at_start() {
    let (root, _home, workspace) = test_home_workspace();
    std::fs::write(workspace.join(".kissconfig"), "k\n").expect("write kissconfig");
    let mock = root.path().join("mock-agent-acp-do-tamper-kiss");
    common::write_mock_executable(&mock, &acp_mock_do_tampers_kissconfig_js_only());
    let out = run_malvin_do_home_workspace(&workspace, &root.path().join("home"), &mock);
    assert!(
        out.status.success(),
        "malvin do failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let restored = std::fs::read_to_string(workspace.join(".kissconfig")).expect("read kissconfig");
    assert_eq!(restored, "k\n");
}

#[cfg_attr(unix, test)]
fn do_restores_workspace_kissignore_after_mock_agent_overwrites() {
    let (root, home, workspace) = test_home_workspace();
    std::fs::write(workspace.join(".kissignore"), "x\n").expect("write .kissignore");
    let mock = root.path().join("mock-agent-acp-do-kissignore");
    common::write_mock_executable(&mock, &acp_mock_do_tampers_kissignore_js());
    let out = run_malvin_do_home_workspace(&workspace, &home, &mock);
    assert!(
        out.status.success(),
        "malvin do failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let restored =
        std::fs::read_to_string(workspace.join(".kissignore")).expect("read .kissignore");
    assert_eq!(restored, "x\n");
}

#[cfg_attr(unix, test)]
fn do_restores_missing_kissignore_when_agent_creates_it() {
    let (root, _home, workspace) = test_home_workspace();
    let _ = std::fs::remove_file(workspace.join(".kissignore"));
    let mock = root.path().join("mock-agent-acp-do-create-kissignore");
    common::write_mock_executable(&mock, &acp_mock_do_creates_kissignore_js());
    let out = run_malvin_do_home_workspace(&workspace, &root.path().join("home"), &mock);
    assert!(
        out.status.success(),
        "malvin do failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!workspace.join(".kissignore").exists());
}

#[cfg_attr(unix, test)]
fn do_restores_kissignore_after_tamper_when_present_at_start() {
    let (root, _home, workspace) = test_home_workspace();
    std::fs::write(workspace.join(".kissignore"), "i\n").expect("write .kissignore");
    let mock = root.path().join("mock-agent-acp-do-tamper-kissignore");
    common::write_mock_executable(&mock, &acp_mock_do_tampers_kissignore_js_only());
    let out = run_malvin_do_home_workspace(&workspace, &root.path().join("home"), &mock);
    assert!(
        out.status.success(),
        "malvin do failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let restored =
        std::fs::read_to_string(workspace.join(".kissignore")).expect("read .kissignore");
    assert_eq!(restored, "i\n");
}

#[cfg_attr(unix, test)]
fn do_stdout_omits_outgoing_prompt_bracket_line() {
    let out = run_do_with_mock_force_tee(&[]);
    assert!(out.status.success(), "malvin do failed: {out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let who = malvin::output::format_acp_directional_tag_prefix('>', "do");
    let inner = malvin::output::format_log_tag_inner(&who);
    assert!(
        !stdout.contains(&format!("[{inner}] [do...]")),
        "forced tee must not print padded >do bracket line on stdout: {stdout:?}"
    );
    assert!(
        !stdout.contains("[do...]"),
        "forced tee must not print outgoing prompt bracket on stdout: {stdout:?}"
    );
    assert!(
        !stdout.contains(">do"),
        "forced tee must not print >do stem on stdout: {stdout:?}"
    );
    assert!(
        stdout.contains("agent message"),
        "expected agent output on stdout: {stdout:?}"
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
    let (out, root, workspace) = run_do_with_named_mock_bin(
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
    let log_path = first_do_log_path(&workspace, &root.path().join("home"));
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
