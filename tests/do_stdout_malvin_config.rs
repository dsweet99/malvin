mod common;

#[cfg(unix)]
use common::{
    acp_mock_do_creates_malvin_config_js, acp_mock_do_tampers_malvin_config_js,
    acp_mock_do_tampers_malvin_config_js_only, run_malvin_do_home_workspace, test_home_workspace,
    cached_mock_executable,
};

#[cfg(unix)]
fn home_config_path(home: &std::path::Path) -> std::path::PathBuf {
    home.join(".malvin_home/config.toml")
}

#[cfg(unix)]
const HOME_CONFIG_SEED: &str = "mem_limit_gb = 7\n";

#[cfg_attr(unix, test)]
fn do_restores_home_malvin_config_after_mock_agent_overwrites() {
    let (_root, home, workspace) = test_home_workspace();
    common::activate_test_home(&home);
    common::seed_malvin_config(&workspace, HOME_CONFIG_SEED);
    let mock = cached_mock_executable( &acp_mock_do_tampers_malvin_config_js());
    let out = run_malvin_do_home_workspace(&workspace, &home, &mock);
    assert!(
        out.status.success(),
        "malvin do failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let restored = std::fs::read_to_string(home_config_path(&home)).expect("read ~/.malvin_home/config.toml");
    assert!(
        restored.contains("mem_limit_gb = 7"),
        "expected restored home config, got: {restored:?}"
    );
    assert!(
        !restored.contains("TAMPERED"),
        "agent tamper must not persist: {restored:?}"
    );
}

#[cfg_attr(unix, test)]
fn do_restores_missing_malvin_config_when_agent_creates_it() {
    let (_root, home, workspace) = test_home_workspace();
    common::activate_test_home(&home);
    let cfg = home_config_path(&home);
    let _ = cfg.parent().map(std::fs::remove_dir_all);
    let mock = cached_mock_executable( &acp_mock_do_creates_malvin_config_js());
    let out = run_malvin_do_home_workspace(&workspace, &home, &mock);
    assert!(
        out.status.success(),
        "malvin do failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let restored = std::fs::read_to_string(&cfg).expect("read ~/.malvin_home/config.toml");
    assert!(
        restored.contains("[agent]"),
        "expected default home config after restore, got: {restored:?}"
    );
    assert!(
        !restored.contains("CREATED"),
        "agent-created tamper must not persist: {restored:?}"
    );
}

#[cfg_attr(unix, test)]
fn do_restores_malvin_config_after_tamper_when_present_at_start() {
    let (_root, home, workspace) = test_home_workspace();
    common::activate_test_home(&home);
    common::seed_malvin_config(&workspace, "mem_limit_gb = 3\n");
    let mock = cached_mock_executable( &acp_mock_do_tampers_malvin_config_js_only());
    let out = run_malvin_do_home_workspace(&workspace, &home, &mock);
    assert!(
        out.status.success(),
        "malvin do failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let restored = std::fs::read_to_string(home_config_path(&home)).expect("read ~/.malvin_home/config.toml");
    assert!(
        restored.contains("mem_limit_gb = 3"),
        "expected restored home config, got: {restored:?}"
    );
    assert!(
        !restored.contains("TAMPERED"),
        "agent tamper must not persist: {restored:?}"
    );
}
