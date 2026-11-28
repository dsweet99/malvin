#[cfg(test)]
use std::path::Path;

#[cfg(test)]
pub fn seed_malvin_checks(work: &Path, content: &str) {
    crate::test_support::test_utils::ensure_git_repo(work);
    let path = crate::malvin_checks_path(work);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir checks parent");
    }
    std::fs::write(path, content).expect("write checks");
}

#[cfg(test)]
pub fn seed_malvin_config(content: &str) {
    assert!(
        crate::workspace::workspace_paths::home_malvin_config_disk_io_allowed(),
        "seed_malvin_config requires with_isolated_home or activate_test_home (see plan.md)"
    );
    let path = crate::malvin_home_config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir ~/.malvinconf");
    }
    std::fs::write(path, content).expect("write ~/.malvinconf/config.toml");
}

#[cfg(test)]
#[path = "malvin_test_seed_tests.rs"]
mod malvin_test_seed_tests;
