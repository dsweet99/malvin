mod common;

#[cfg(unix)]
use common::{
    acp_mock_do_creates_malvin_config_js, acp_mock_do_tampers_malvin_config_js,
    acp_mock_do_tampers_malvin_config_js_only, run_malvin_do_home_workspace, test_home_workspace,
    write_mock_executable,
};

#[cfg_attr(unix, test)]
fn do_restores_workspace_malvin_config_after_mock_agent_overwrites() {
    let (root, home, workspace) = test_home_workspace();
    common::seed_malvin_config(&workspace, "x\n");
    let mock = root.path().join("mock-agent-acp-do-malvin-config");
    write_mock_executable(&mock, &acp_mock_do_tampers_malvin_config_js());
    let out = run_malvin_do_home_workspace(&workspace, &home, &mock);
    assert!(
        out.status.success(),
        "malvin do failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let restored = std::fs::read_to_string(workspace.join(".malvin/config.toml"))
        .expect("read .malvin/config.toml");
    assert_eq!(restored, "x\n");
}

#[cfg_attr(unix, test)]
fn do_restores_missing_malvin_config_when_agent_creates_it() {
    let (root, _home, workspace) = test_home_workspace();
    let cfg = workspace.join(".malvin/config.toml");
    let _ = cfg.parent().map(std::fs::remove_dir_all);
    let mock = root.path().join("mock-agent-acp-do-create-malvin-config");
    write_mock_executable(&mock, &acp_mock_do_creates_malvin_config_js());
    let out = run_malvin_do_home_workspace(&workspace, &root.path().join("home"), &mock);
    assert!(
        out.status.success(),
        "malvin do failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!cfg.exists());
}

#[cfg_attr(unix, test)]
fn do_restores_malvin_config_after_tamper_when_present_at_start() {
    let (root, _home, workspace) = test_home_workspace();
    common::seed_malvin_config(&workspace, "cfg\n");
    let mock = root.path().join("mock-agent-acp-do-tamper-malvin-config");
    write_mock_executable(&mock, &acp_mock_do_tampers_malvin_config_js_only());
    let out = run_malvin_do_home_workspace(&workspace, &root.path().join("home"), &mock);
    assert!(
        out.status.success(),
        "malvin do failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let restored = std::fs::read_to_string(workspace.join(".malvin/config.toml"))
        .expect("read .malvin/config.toml");
    assert_eq!(restored, "cfg\n");
}
