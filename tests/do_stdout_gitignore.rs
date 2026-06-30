mod common;

#[cfg(unix)]
use common::{
    acp_mock_do_creates_gitignore_js, acp_mock_do_tampers_gitignore_js,
    acp_mock_do_tampers_gitignore_js_only, run_malvin_do_home_workspace, test_home_workspace,
};

#[cfg_attr(unix, test)]
fn do_restores_gitignore_after_mock_agent_overwrites() {
    let (_root, home, workspace) = test_home_workspace();
    std::fs::write(workspace.join(".gitignore"), "g\n").expect("write .gitignore");
    let mock = common::cached_mock_executable(&acp_mock_do_tampers_gitignore_js());
    let out = run_malvin_do_home_workspace(&workspace, &home, &mock);
    assert!(
        out.status.success(),
        "malvin do failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let restored =
        std::fs::read_to_string(workspace.join(".gitignore")).expect("read .gitignore");
    assert_eq!(restored, "g\n");
}

#[cfg_attr(unix, test)]
fn do_restores_gitignore_after_tamper_when_present_at_start() {
    let (root, _home, workspace) = test_home_workspace();
    std::fs::write(workspace.join(".gitignore"), "gi\n").expect("write .gitignore");
    let mock = common::cached_mock_executable(&acp_mock_do_tampers_gitignore_js_only());
    let out = run_malvin_do_home_workspace(&workspace, &root.path().join("home"), &mock);
    assert!(
        out.status.success(),
        "malvin do failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let restored =
        std::fs::read_to_string(workspace.join(".gitignore")).expect("read .gitignore");
    assert_eq!(restored, "gi\n");
}

#[cfg_attr(unix, test)]
fn do_restores_missing_gitignore_when_agent_creates_it() {
    let (root, _home, workspace) = test_home_workspace();
    let _ = std::fs::remove_file(workspace.join(".gitignore"));
    let mock = common::cached_mock_executable(&acp_mock_do_creates_gitignore_js());
    let out = run_malvin_do_home_workspace(&workspace, &root.path().join("home"), &mock);
    assert!(
        out.status.success(),
        "malvin do failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!workspace.join(".gitignore").exists());
}
