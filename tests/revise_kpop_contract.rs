//! `malvin revise` runs the kpop gate-loop workflow with composed `revise_constraints.md`.

#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    ReviseSpawn, acp_mock_revise_kpop_empty_output_js, acp_mock_revise_agent_ran_without_output_js,
    acp_mock_revise_kpop_steps_js, bin_path_with_fake_kiss, combined_cli_output, fast_test_home_workspace,
    seed_malvin_checks, spawn_revise, cached_mock_executable,
};

#[cfg(unix)]
#[test]
fn revise_succeeds_when_agent_writes_valid_document() {
    let (root, home, workspace) = fast_test_home_workspace();
    seed_malvin_checks(&workspace, "true\n");
    std::fs::write(workspace.join("doc.md"), "# Draft\n\nHedgy maybe text.\n").expect("seed");
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable( &acp_mock_revise_kpop_steps_js());
    let out = spawn_revise(&ReviseSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        doc_path: "doc.md",
        extra_args: &["--max-loops", "0"],
    });
    assert!(
        out.status.success(),
        "revise must succeed when agent writes valid document: {:?}",
        combined_cli_output(&out)
    );
    let doc = std::fs::read_to_string(workspace.join("doc.md")).expect("read doc");
    assert!(!doc.is_empty(), "document must be non-empty");
}

#[cfg(unix)]
#[test]
fn revise_runs_kpop_when_gates_already_pass() {
    let (root, home, workspace) = fast_test_home_workspace();
    seed_malvin_checks(&workspace, "true\n");
    std::fs::write(workspace.join("doc.md"), "# Draft\n\nHedgy maybe text.\n").expect("seed");
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable( &acp_mock_revise_kpop_steps_js());
    let out = spawn_revise(&ReviseSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        doc_path: "doc.md",
        extra_args: &["--max-loops", "0"],
    });
    let combined = combined_cli_output(&out);
    assert!(
        combined.contains("KPOP_LOG:"),
        "revise must run kpop even when gates pass before agent: status={:?} combined={combined:?}",
        out.status,
    );
}

#[cfg(unix)]
#[test]
fn revise_fails_when_doc_path_missing() {
    let (root, home, workspace) = fast_test_home_workspace();
    seed_malvin_checks(&workspace, "true\n");
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable( &acp_mock_revise_kpop_steps_js());
    let out = spawn_revise(&ReviseSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        doc_path: "missing.md",
        extra_args: &["--max-loops", "0"],
    });
    let combined = combined_cli_output(&out);
    assert!(!out.status.success(), "expected failure when doc missing: {combined:?}");
    assert!(
        combined.contains("not an existing file"),
        "expected missing-file error: {combined:?}"
    );
    assert!(
        !combined.contains("KPOP_LOG:"),
        "agent must not run when preflight fails: {combined:?}"
    );
}

#[cfg(unix)]
#[test]
fn revise_fails_when_agent_solves_but_output_empty() {
    let (root, home, workspace) = fast_test_home_workspace();
    seed_malvin_checks(&workspace, "true\n");
    std::fs::write(workspace.join("doc.md"), "seed\n").expect("seed");
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable( &acp_mock_revise_kpop_empty_output_js());
    let out = spawn_revise(&ReviseSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        doc_path: "doc.md",
        extra_args: &["--max-loops", "0"],
    });
    assert!(!out.status.success(), "expected failure for empty doc: {out:?}");
}

#[cfg(unix)]
#[test]
fn revise_fails_when_agent_solves_but_output_missing() {
    let (root, home, workspace) = fast_test_home_workspace();
    seed_malvin_checks(&workspace, "true\n");
    std::fs::write(workspace.join("doc.md"), "seed\n").expect("seed");
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable( &acp_mock_revise_agent_ran_without_output_js());
    let out = spawn_revise(&ReviseSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        doc_path: "doc.md",
        extra_args: &["--max-loops", "0"],
    });
    assert!(!out.status.success(), "expected failure when output missing: {out:?}");
}
