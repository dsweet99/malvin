//! `malvin explain` is a router-backed request wrapper.

#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    ExplainSpawn, acp_mock_router_no_work_js, bin_path_with_fake_kiss, combined_cli_output,
    seed_git_kiss_cargo_gate_workspace, spawn_explain, test_home_workspace,
    workspace_kiss_check_only, cached_mock_executable,
};

#[cfg(unix)]
#[test]
fn explain_router_succeeds_with_mock() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable(&acp_mock_router_no_work_js());
    let out = spawn_explain(&ExplainSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        request: "gate loop exit",
        extra_args: &["--max-loops", "1"],
    });
    assert!(
        out.status.success(),
        "explain must succeed via router: {:?}",
        combined_cli_output(&out)
    );
    let tex = std::fs::metadata(workspace.join("explain.tex")).expect("tex exists");
    let pdf = std::fs::metadata(workspace.join("explain.pdf")).expect("pdf exists");
    assert!(tex.len() > 0 && pdf.len() > 0, "composed explain request should yield tex/pdf");
}

#[cfg(unix)]
#[test]
fn explain_embeds_user_request_in_router_request() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable(&acp_mock_router_no_work_js());
    let out = spawn_explain(&ExplainSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        request: "unique-explain-topic-xyz",
        extra_args: &["--max-loops", "1"],
    });
    let combined = combined_cli_output(&out);
    assert!(out.status.success(), "explain must succeed: {combined:?}");
    assert!(
        combined.contains("unique-explain-topic-xyz") || combined.contains("User request:"),
        "composed request must embed user request: {combined:?}"
    );
}

#[cfg(unix)]
#[test]
fn explain_fails_when_request_missing() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable(&acp_mock_router_no_work_js());
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.current_dir(&workspace)
        .env("HOME", &home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", &mock)
        .env("PATH", &path)
        .args(["explain", "--max-loops", "1"]);
    let out = common::command_output_with_timeout(&mut cmd, common::MALVIN_TEST_CMD_TIMEOUT)
        .expect("spawn");
    let combined = combined_cli_output(&out);
    assert!(
        out.status.success() || combined.contains("REQUEST") || combined.contains("Usage"),
        "bare explain should help or error without router: {combined:?}"
    );
    assert!(
        !combined.contains("router_requirements"),
        "missing request must not start router work: {combined:?}"
    );
}

#[cfg(unix)]
#[test]
fn explain_fails_when_custom_out_path_preexists() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    std::fs::create_dir_all(workspace.join("docs")).expect("mkdir");
    std::fs::write(workspace.join("docs/paper.tex"), "stale\n").expect("seed");
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable(&acp_mock_router_no_work_js());
    let out = spawn_explain(&ExplainSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        request: "topic",
        extra_args: &["--max-loops", "1", "--out-path", "docs/paper.tex"],
    });
    let combined = combined_cli_output(&out);
    assert!(!out.status.success(), "expected overwrite refusal: {combined:?}");
    assert!(
        combined.contains("refusing to overwrite"),
        "expected overwrite refusal: {combined:?}"
    );
}
