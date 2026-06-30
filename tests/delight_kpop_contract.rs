//! `malvin delight` runs the kpop gate-loop workflow with composed `delight_constraints.md`.

#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    DelightSpawn, acp_mock_delight_kpop_empty_output_js, acp_mock_delight_kpop_solved_without_output_js,
    acp_mock_delight_kpop_steps_js, bin_path_with_fake_kiss, combined_cli_output,
    seed_git_kiss_cargo_gate_workspace, spawn_delight, test_home_workspace, workspace_kiss_check_only,
    cached_mock_executable,
};

#[cfg(unix)]
#[test]
fn delight_runs_kpop_when_gates_already_pass() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable( &acp_mock_delight_kpop_steps_js());
    let out = spawn_delight(&DelightSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "1"],
    });
    let combined = combined_cli_output(&out);
    assert!(
        combined.contains("KPOP_LOG:"),
        "delight must run kpop even when gates pass before agent: status={:?} combined={combined:?}",
        out.status,
    );
}

#[cfg(unix)]
#[test]
fn delight_allocates_sibling_when_default_pitch_preexists() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    std::fs::write(workspace.join("pitch.md"), "existing\n").expect("seed pitch");
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable( &acp_mock_delight_kpop_steps_js());
    let out = spawn_delight(&DelightSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "1"],
    });
    let combined = combined_cli_output(&out);
    assert!(
        combined.contains("KPOP_LOG:"),
        "delight must run kpop after sibling allocation: status={:?} combined={combined:?}",
        out.status,
    );
    let stale = std::fs::read_to_string(workspace.join("pitch.md")).expect("read stale pitch");
    assert_eq!(stale, "existing\n", "original pitch.md must be untouched");
    assert!(
        workspace.join("pitch_1.md").exists(),
        "preflight must allocate pitch_1.md before kpop starts"
    );
}

#[cfg(unix)]
#[test]
fn delight_fails_when_custom_out_path_preexists() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    std::fs::create_dir_all(workspace.join("plans")).expect("mkdir");
    std::fs::write(workspace.join("plans/existing.md"), "existing\n").expect("seed plan");
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable( &acp_mock_delight_kpop_steps_js());
    let out = spawn_delight(&DelightSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "1", "--out-path", "plans/existing.md"],
    });
    let combined = combined_cli_output(&out);
    assert!(!out.status.success(), "expected failure when custom path exists: {combined:?}");
    assert!(
        combined.contains("refusing to overwrite"),
        "expected overwrite refusal: {combined:?}"
    );
    assert!(
        !combined.contains("KPOP_LOG:"),
        "agent must not run when preflight fails: {combined:?}"
    );
}

#[cfg(unix)]
#[test]
fn delight_fails_when_agent_solves_but_output_missing() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable( &acp_mock_delight_kpop_solved_without_output_js());
    let out = spawn_delight(&DelightSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "1"],
    });
    assert!(!out.status.success(), "expected failure when output missing: {out:?}");
}

#[cfg(unix)]
#[test]
fn delight_writes_custom_out_path() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable( &acp_mock_delight_kpop_steps_js());
    let out = spawn_delight(&DelightSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "1", "--out-path", "plans/new.md"],
    });
    let combined = combined_cli_output(&out);
    assert!(
        combined.contains("KPOP_LOG:"),
        "delight with custom out-path must enter kpop gate loop: status={:?} combined={combined:?}",
        out.status,
    );
    assert!(
        !workspace.join("pitch.md").exists(),
        "default pitch.md must not be created when out-path is custom"
    );
}

#[cfg(unix)]
#[test]
fn delight_kpop_fails_when_post_session_output_empty() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable( &acp_mock_delight_kpop_empty_output_js());
    let out = spawn_delight(&DelightSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "1"],
    });
    assert!(!out.status.success(), "expected failure for empty output: {out:?}");
}
