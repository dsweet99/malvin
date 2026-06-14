//! `malvin explain` runs the kpop gate-loop workflow with composed `explain_constraints.md`.

#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    ExplainSpawn, acp_mock_explain_kpop_empty_pdf_js, acp_mock_explain_kpop_solved_without_output_js,
    acp_mock_explain_kpop_steps_js, bin_path_with_fake_kiss, combined_cli_output,
    seed_git_kiss_cargo_gate_workspace, seed_stale_default_explain_outputs,
    spawn_explain, test_home_workspace, workspace_kiss_check_only, write_mock_executable,
};

#[cfg(unix)]
#[test]
fn explain_runs_kpop_when_gates_already_pass() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let path = bin_path_with_fake_kiss(&root);
    let mock = root.path().join("mock-explain-kpop");
    write_mock_executable(&mock, &acp_mock_explain_kpop_steps_js());
    let out = spawn_explain(&ExplainSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        request: "gate loop exit",
        extra_args: &["--max-loops", "1"],
    });
    let combined = combined_cli_output(&out);
    assert!(
        combined.contains("KPOP_LOG:"),
        "explain must run kpop even when gates pass before agent: status={:?} combined={combined:?}",
        out.status,
    );
}

#[cfg(unix)]
#[test]
fn explain_writes_custom_out_path() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let path = bin_path_with_fake_kiss(&root);
    let mock = root.path().join("mock-explain-custom-out");
    write_mock_executable(&mock, &acp_mock_explain_kpop_steps_js());
    let out = spawn_explain(&ExplainSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        request: "gate loop exit",
        extra_args: &["--max-loops", "1", "--out-path", "docs/paper.tex"],
    });
    let combined = combined_cli_output(&out);
    assert!(
        combined.contains("KPOP_LOG:"),
        "explain with custom out-path must enter kpop gate loop: status={:?} combined={combined:?}",
        out.status,
    );
    assert!(
        !workspace.join("explain.tex").exists(),
        "default explain.tex must not be created when out-path is custom"
    );
}

#[cfg(unix)]
#[test]
fn explain_fails_when_request_missing() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let path = bin_path_with_fake_kiss(&root);
    let mock = root.path().join("mock-explain-should-not-run");
    write_mock_executable(&mock, &acp_mock_explain_kpop_steps_js());
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.current_dir(&workspace)
        .env("HOME", &home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", &mock)
        .env("PATH", &path)
        .args(["explain", "--max-loops", "1"]);
    let out = common::command_output_with_timeout(&mut cmd, common::MALVIN_TEST_CMD_TIMEOUT)
        .expect("spawn");
    assert!(out.status.success(), "bare explain prints short help");
    let combined = combined_cli_output(&out);
    assert!(
        !combined.contains("KPOP_LOG:"),
        "agent must not run when request missing: {combined:?}"
    );
}

#[cfg(unix)]
#[test]
fn explain_auto_mode_leaves_stale_default_outputs_untouched() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    seed_stale_default_explain_outputs(&workspace);
    let path = bin_path_with_fake_kiss(&root);
    let mock = root.path().join("mock-explain-stale");
    write_mock_executable(&mock, &acp_mock_explain_kpop_steps_js());
    let out = spawn_explain(&ExplainSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        request: "topic",
        extra_args: &["--max-loops", "1"],
    });
    let combined = combined_cli_output(&out);
    assert!(
        combined.contains("KPOP_LOG:"),
        "explain must run kpop in auto out-path mode: status={:?} combined={combined:?}",
        out.status,
    );
    let stale = std::fs::read_to_string(workspace.join("explain.tex")).expect("read stale tex");
    assert_eq!(stale, "STALE\n", "original explain.tex must be untouched");
    assert!(
        workspace.join("gate_loop_exit.tex").exists(),
        "auto mode must discover agent-written title-based output"
    );
}

#[cfg(unix)]
#[test]
fn explain_fails_when_agent_solves_but_output_missing() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let path = bin_path_with_fake_kiss(&root);
    let mock = root.path().join("mock-explain-no-output");
    write_mock_executable(&mock, &acp_mock_explain_kpop_solved_without_output_js());
    let out = spawn_explain(&ExplainSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        request: "topic",
        extra_args: &["--max-loops", "1"],
    });
    assert!(!out.status.success(), "expected failure when output missing: {out:?}");
}

#[cfg(unix)]
#[test]
fn explain_kpop_fails_when_post_session_pdf_empty() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let path = bin_path_with_fake_kiss(&root);
    let mock = root.path().join("mock-explain-empty-pdf");
    write_mock_executable(&mock, &acp_mock_explain_kpop_empty_pdf_js());
    let out = spawn_explain(&ExplainSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        request: "topic",
        extra_args: &["--max-loops", "1"],
    });
    assert!(!out.status.success(), "expected failure for empty pdf: {out:?}");
}
