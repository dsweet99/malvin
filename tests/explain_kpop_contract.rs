//! `malvin explain` runs the kpop gate-loop workflow with composed `explain_constraints.md`.

#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    ExplainSpawn, acp_mock_explain_kpop_empty_pdf_js, acp_mock_explain_kpop_solved_without_output_js,
    acp_mock_explain_kpop_steps_js, bin_path_with_fake_kiss, combined_cli_output,
    seed_git_kiss_cargo_gate_workspace, spawn_explain, test_home_workspace, workspace_kiss_check_only,
    write_mock_executable,
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
        out.status.success(),
        "expected explain success: status={:?} combined={combined:?}",
        out.status,
    );
    assert!(combined.contains("DONE"), "expected DONE: {combined:?}");
    assert!(
        combined.contains("KPOP_LOG:"),
        "explain must run kpop even when gates pass before agent: {combined:?}"
    );
    let tex = std::fs::read_to_string(workspace.join("explain.tex")).expect("read tex");
    assert!(!tex.is_empty(), "output tex must be non-empty");
    assert!(
        tex.contains("Revised"),
        "explain must chain malvin revise on success: {tex:?}"
    );
    let pdf = std::fs::read(workspace.join("explain.pdf")).expect("read pdf");
    assert!(!pdf.is_empty(), "output pdf must be non-empty");
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
        out.status.success(),
        "expected explain success with custom out-path: status={:?} combined={combined:?}",
        out.status,
    );
    let tex = std::fs::read_to_string(workspace.join("docs/paper.tex")).expect("read tex");
    assert!(!tex.is_empty(), "custom tex must be non-empty");
    let pdf = std::fs::read(workspace.join("docs/paper.pdf")).expect("read pdf");
    assert!(!pdf.is_empty(), "custom pdf must be non-empty");
    assert!(
        !workspace.join("explain.tex").exists(),
        "default explain.tex must not be written when out-path is custom"
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
fn explain_fails_when_stale_outputs_exist() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    std::fs::write(workspace.join("explain.tex"), "STALE\n").expect("write");
    std::fs::write(workspace.join("explain.pdf"), b"%PDF-1.4 stale").expect("write");
    let path = bin_path_with_fake_kiss(&root);
    let mock = root.path().join("mock-explain-stale");
    write_mock_executable(&mock, &acp_mock_explain_kpop_solved_without_output_js());
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
        !out.status.success(),
        "expected failure when stale outputs exist: status={:?} combined={combined:?}",
        out.status,
    );
    assert!(
        combined.contains("refusing to overwrite"),
        "expected overwrite refusal: {combined:?}"
    );
    let tex = std::fs::read_to_string(workspace.join("explain.tex")).expect("read tex");
    assert_eq!(tex, "STALE\n", "stale tex must be unchanged");
    assert!(
        !combined.contains("KPOP_LOG:"),
        "agent must not run when preflight fails: {combined:?}"
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
