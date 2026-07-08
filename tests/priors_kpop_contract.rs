//! `malvin priors` runs the kpop gate-loop workflow with composed `priors_constraints.md`.

#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    PriorsSpawn, acp_mock_priors_kpop_empty_output_js, acp_mock_priors_agent_ran_without_output_js,
    acp_mock_priors_kpop_steps_js, bin_path_with_fake_kiss, combined_cli_output, seed_malvin_checks,
    spawn_priors, fast_test_home_workspace, cached_mock_executable,
};

#[cfg(unix)]
#[test]
fn priors_succeeds_when_agent_writes_valid_report() {
    let (root, home, workspace) = fast_test_home_workspace();
    seed_malvin_checks(&workspace, "true\n");
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable(&acp_mock_priors_kpop_steps_js());
    let out = spawn_priors(&PriorsSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "0"],
    });
    assert!(
        out.status.success(),
        "priors must succeed when agent writes valid report: {:?}",
        combined_cli_output(&out)
    );
    let report = std::fs::read_to_string(workspace.join("priors.md")).expect("read priors");
    assert!(!report.is_empty(), "priors report must be non-empty");
}

#[cfg(unix)]
#[test]
fn priors_runs_kpop_when_gates_already_pass() {
    let (root, home, workspace) = fast_test_home_workspace();
    seed_malvin_checks(&workspace, "true\n");
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable(&acp_mock_priors_kpop_steps_js());
    let out = spawn_priors(&PriorsSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "0"],
    });
    let combined = combined_cli_output(&out);
    assert!(
        combined.contains("KPOP_LOG:"),
        "priors must run kpop even when gates pass before agent: status={:?} combined={combined:?}",
        out.status,
    );
}

#[cfg(unix)]
#[test]
fn priors_allocates_sibling_when_default_priors_preexists() {
    let (root, home, workspace) = fast_test_home_workspace();
    seed_malvin_checks(&workspace, "true\n");
    std::fs::write(workspace.join("priors.md"), "existing\n").expect("seed priors");
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable(&acp_mock_priors_kpop_steps_js());
    let out = spawn_priors(&PriorsSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "0"],
    });
    let combined = combined_cli_output(&out);
    assert!(
        combined.contains("KPOP_LOG:"),
        "priors must run kpop after sibling allocation: status={:?} combined={combined:?}",
        out.status,
    );
    let stale = std::fs::read_to_string(workspace.join("priors.md")).expect("read stale priors");
    assert_eq!(stale, "existing\n", "original priors.md must be untouched");
    assert!(
        workspace.join("priors_1.md").exists(),
        "preflight must allocate priors_1.md before kpop starts"
    );
}

#[cfg(unix)]
#[test]
fn priors_fails_when_custom_out_path_preexists() {
    let (root, home, workspace) = fast_test_home_workspace();
    seed_malvin_checks(&workspace, "true\n");
    std::fs::create_dir_all(workspace.join("reports")).expect("mkdir");
    std::fs::write(workspace.join("reports/existing.md"), "existing\n").expect("seed report");
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable(&acp_mock_priors_kpop_steps_js());
    let out = spawn_priors(&PriorsSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "0", "--out-path", "reports/existing.md"],
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
fn priors_fails_when_agent_solves_but_output_missing() {
    let (root, home, workspace) = fast_test_home_workspace();
    seed_malvin_checks(&workspace, "true\n");
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable(&acp_mock_priors_agent_ran_without_output_js());
    let out = spawn_priors(&PriorsSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "0"],
    });
    assert!(!out.status.success(), "expected failure when output missing: {out:?}");
}

#[cfg(unix)]
#[test]
fn priors_writes_custom_out_path() {
    let (root, home, workspace) = fast_test_home_workspace();
    seed_malvin_checks(&workspace, "true\n");
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable(&acp_mock_priors_kpop_steps_js());
    let out = spawn_priors(&PriorsSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "0", "--out-path", "reports/new.md"],
    });
    let combined = combined_cli_output(&out);
    assert!(
        combined.contains("KPOP_LOG:"),
        "priors with custom out-path must enter kpop gate loop: status={:?} combined={combined:?}",
        out.status,
    );
    assert!(
        !workspace.join("priors.md").exists(),
        "default priors.md must not be created when out-path is custom"
    );
}

#[cfg(unix)]
#[test]
fn priors_kpop_fails_when_post_session_output_empty() {
    let (root, home, workspace) = fast_test_home_workspace();
    seed_malvin_checks(&workspace, "true\n");
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable(&acp_mock_priors_kpop_empty_output_js());
    let out = spawn_priors(&PriorsSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "0"],
    });
    assert!(!out.status.success(), "expected failure for empty output: {out:?}");
}
