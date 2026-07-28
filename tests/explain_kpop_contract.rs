//! `malvin explain` runs Review (`KPop`) → Plan (`KPop`) → Work until chat LGTM.

#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    ExplainSpawn, acp_mock_explain_kpop_empty_pdf_js, acp_mock_explain_agent_ran_without_output_js,
    acp_mock_explain_kpop_steps_js, acp_mock_explain_lgtm_first_review_js, bin_path_with_fake_kiss,
    combined_cli_output, seed_git_kiss_cargo_gate_workspace, seed_stale_default_explain_outputs,
    spawn_explain, test_home_workspace, workspace_kiss_check_only, cached_mock_executable,
};

#[cfg(unix)]
#[test]
fn explain_succeeds_when_agent_writes_valid_tex_and_pdf() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable(&acp_mock_explain_kpop_steps_js());
    let out = spawn_explain(&ExplainSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        request: "gate loop exit",
        extra_args: &["--max-loops", "2"],
    });
    assert!(
        out.status.success(),
        "explain must succeed when agent writes valid tex and pdf: {:?}",
        combined_cli_output(&out)
    );
    let tex = std::fs::metadata(workspace.join("gate_loop_exit.tex")).expect("tex exists");
    let pdf = std::fs::metadata(workspace.join("gate_loop_exit.pdf")).expect("pdf exists");
    assert!(tex.len() > 0, "tex must be non-empty");
    assert!(pdf.len() > 0, "pdf must be non-empty");
}

#[cfg(unix)]
#[test]
fn explain_runs_kpop_when_gates_already_pass() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable(&acp_mock_explain_kpop_steps_js());
    let out = spawn_explain(&ExplainSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        request: "gate loop exit",
        extra_args: &["--max-loops", "2"],
    });
    let combined = combined_cli_output(&out);
    assert!(
        combined.contains("KPOP_LOG:"),
        "explain must run kpop review/plan even when gates pass before agent: status={:?} combined={combined:?}",
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
    let mock = cached_mock_executable(&acp_mock_explain_kpop_steps_js());
    let out = spawn_explain(&ExplainSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        request: "gate loop exit",
        extra_args: &["--max-loops", "2", "--out-path", "docs/paper.tex"],
    });
    let combined = combined_cli_output(&out);
    assert!(
        combined.contains("KPOP_LOG:"),
        "explain with custom out-path must enter review/plan kpop: status={:?} combined={combined:?}",
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
    let mock = cached_mock_executable(&acp_mock_explain_kpop_steps_js());
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.current_dir(&workspace)
        .env("HOME", &home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", &mock)
        .env("PATH", &path)
        .args(common::INTEGRATION_TEST_MALVIN_ARGS)
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
    let mock = cached_mock_executable(&acp_mock_explain_kpop_steps_js());
    let out = spawn_explain(&ExplainSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        request: "topic",
        extra_args: &["--max-loops", "2"],
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
    let mock = cached_mock_executable(&acp_mock_explain_agent_ran_without_output_js());
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
    let mock = cached_mock_executable(&acp_mock_explain_kpop_empty_pdf_js());
    // Empty-PDF mock walks review→plan→work→review across max-loops=2; needs >12s.
    let out = common::spawn_explain_with_timeout(
        &ExplainSpawn {
            workspace: &workspace,
            home: &home,
            mock: &mock,
            path_var: &path,
            request: "topic",
            extra_args: &["--max-loops", "2"],
        },
        std::time::Duration::from_secs(25),
    );
    assert!(!out.status.success(), "expected failure for empty pdf: {out:?}");
}

#[cfg(unix)]
#[test]
fn explain_lgtm_on_first_review_skips_plan_and_work() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable(&acp_mock_explain_lgtm_first_review_js());
    let out = spawn_explain(&ExplainSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        request: "topic",
        extra_args: &["--max-loops", "1", "--out-path", "docs/paper.tex"],
    });
    let combined = combined_cli_output(&out);
    assert!(
        out.status.success(),
        "LGTM on first review must succeed without plan/work: {combined:?}"
    );
    assert!(
        combined.contains("KPOP_LOG:"),
        "review kpop must still run: {combined:?}"
    );
}
