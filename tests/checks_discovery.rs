//! Deprecated `malvin code` checks-discovery tests.

#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    assert_code_deprecated, spawn_malvin_code_discovery, CodeDiscoverySpawn,
    fast_test_home_workspace,
};

#[cfg(unix)]
#[test]
fn malvin_code_is_deprecated_before_checks_discovery() {
    let (root, home, workspace) = fast_test_home_workspace();
    let path = format!(
        "{}:{}",
        root.path().join("bin").display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let out = spawn_malvin_code_discovery(&CodeDiscoverySpawn {
        project: &workspace,
        home: &home,
        mock_js: "process.exit(0);",
        path_var: &path,
        request: "plan.md",
    });
    assert_code_deprecated(&out);
}
