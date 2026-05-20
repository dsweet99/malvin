//! Behavioral smokes for CLI kiss coverage (static refs live in `cli_cross_cov_kiss.rs`).

#[test]
fn smoke_cov_cli_cross_refs() {
    let _ = super::entrypoint::require_kiss_for_cli_command;
    let _ = crate::cli::entrypoint;
    let _ = crate::cli::run_code;
    let _ = crate::cli::run_do;
    let _ = crate::cli::run_tidy;
    let _ = crate::cli::run_plan;
    let tmp = tempfile::tempdir().unwrap();
    assert!(!crate::source_detect::has_source_files(tmp.path()));
    assert_eq!(
        crate::acp_post_run::merge_acp_and_timing_results(Ok(()), Ok(())),
        Ok(())
    );
    assert_eq!(
        crate::acp_post_run::prefer_primary_over_secondary(Ok(()), Ok(()), "smoke"),
        Ok(())
    );
}

#[cfg(unix)]
#[test]
fn smoke_has_source_files_detects_symlink_to_rs_in_workspace() {
    use std::os::unix::fs::symlink;
    let tmp = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let real = outside.path().join("real.rs");
    std::fs::write(&real, "fn main() {}").unwrap();
    symlink(&real, tmp.path().join("linked.rs")).unwrap();
    assert!(crate::source_detect::has_source_files(tmp.path()));
}

#[test]
fn tidy_zero_max_loops_effective_budget_is_one() {
    assert_eq!(super::tidy_flow::effective_tidy_max_loops(0), 1);
    assert_eq!(super::tidy_flow::effective_tidy_max_loops(3), 3);
}
