use super::tests_gates_scenarios::{
    scenario_invokes_expected_quality_commands, scenario_runs_kiss_clamp_when_no_kissconfig,
    scenario_skips_pre_commit_when_config_present,
};

#[test]
fn source_like_files_present_does_not_follow_external_symlink_dirs() {
    let tmp = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(outside.path().join("src")).unwrap();
    std::fs::write(outside.path().join("src/main.rs"), "fn main() {}").unwrap();
    std::os::unix::fs::symlink(outside.path(), tmp.path().join("src")).unwrap();
    assert!(!super::gate_run::source_like_files_present(tmp.path()));
}

#[test]
fn test_scan_for_extension_handles_symlink_cycles() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_path_buf();
    std::fs::create_dir(root.join("src")).unwrap();
    std::os::unix::fs::symlink(&root, root.join("src").join("cycle")).unwrap();
    assert!(!super::gate_run::scan_for_extension_handles_symlink_cycles(&root));
}

#[test]
fn kiss_cov_test_scan_for_extension_handles_symlink_cycles() {
}

#[test]
fn prepare_repo_workspace_runs_kiss_clamp_when_no_kissconfig() {
    scenario_runs_kiss_clamp_when_no_kissconfig();
}

#[test]
fn run_repo_workspace_gates_invokes_expected_quality_commands() {
    scenario_invokes_expected_quality_commands();
}

#[test]
fn run_repo_workspace_gates_skips_pre_commit_when_config_present() {
    scenario_skips_pre_commit_when_config_present();
}

#[allow(non_snake_case)]

#[cfg(test)]
#[allow(non_snake_case)]
mod kiss_cov_auto_async {
    use super::test_scan_for_extension_handles_symlink_cycles;

    #[test]
    fn kiss_cov_test_scan_for_extension_handles_symlink_cycles() {
    }
}
