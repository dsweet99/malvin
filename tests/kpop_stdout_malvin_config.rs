//! Bare `malvin kpop` preserves `~/.malvin_home/config.toml` across outer-loop restores.

#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    acp_mock_kpop_tampers_home_malvin_config_writes_solved_js, bin_path_with_fake_kiss,
    seed_git_kiss_cargo_gate_workspace, spawn_kpop, test_home_workspace, workspace_kiss_check_only,
    cached_mock_executable, KpopSpawn,
};

#[cfg(unix)]
const HOME_CONFIG_SEED: &str = "mem_limit_gb = 7\nmpc = false\n";

#[cfg(unix)]
fn home_config_path(home: &std::path::Path) -> std::path::PathBuf {
    home.join(".malvin_home/config.toml")
}

#[cfg_attr(unix, test)]
fn kpop_outer_loop_restores_home_malvin_config_after_agent_tampers() {
    let (root, home, workspace) = test_home_workspace();
    common::activate_test_home(&home);
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    common::seed_malvin_config(&workspace, HOME_CONFIG_SEED);
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable( &acp_mock_kpop_tampers_home_malvin_config_writes_solved_js());
    let out = spawn_kpop(&KpopSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "1"],
        request: "investigate config restore",
    });
    assert!(
        out.status.success(),
        "malvin kpop failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let restored =
        std::fs::read_to_string(home_config_path(&home)).expect("read ~/.malvin_home/config.toml");
    assert!(
        restored.contains("mem_limit_gb = 7"),
        "expected restored home config to keep seeded value, got: {restored:?}"
    );
    assert!(
        !restored.contains("TAMPERED"),
        "agent tamper must not persist after kpop outer-loop restore: {restored:?}"
    );
}
