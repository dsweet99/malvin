//! Test helpers for `.malvin/checks` (keeps `test_utils` under kiss line limits).

#[cfg(test)]
use std::path::Path;

#[cfg(test)]
pub fn seed_malvin_checks(work: &Path, content: &str) {
    std::fs::create_dir_all(work.join(crate::MALVIN_DIR)).expect("mkdir .malvin");
    std::fs::write(crate::malvin_checks_path(work), content).expect("write .malvin/checks");
}

/// Requires isolated `HOME`; see plan.md.
#[cfg(test)]
pub fn seed_malvin_config(work: &Path, content: &str) {
    assert!(
        crate::workspace_paths::home_malvin_config_disk_io_allowed(),
        "seed_malvin_config requires with_isolated_home or activate_test_home (see plan.md)"
    );
    let path = crate::malvin_config_path(work);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir ~/.malvin_home");
    }
    std::fs::write(path, content).expect("write ~/.malvin_home/config.toml");
}

#[cfg(test)]
#[path = "malvin_test_seed_tests.rs"]
mod malvin_test_seed_tests;
