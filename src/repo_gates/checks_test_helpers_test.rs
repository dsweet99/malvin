use super::checks_test_helpers::{git_init, write_git_root_checks, write_legacy_cwd_checks};

#[test]
fn kiss_cov_checks_test_helpers_execute() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path();
    git_init(w);
    write_git_root_checks(w, "kiss\n");
    write_legacy_cwd_checks(w, "legacy\n");
    let _ = stringify!(write_legacy_cwd_checks);
}
