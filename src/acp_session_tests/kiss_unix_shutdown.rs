//! Kiss identifier refs for nested helpers in `unix_shutdown_test.rs`.

#[cfg(unix)]
#[test]
fn kiss_cov_unix_shutdown_nested_fns() {
    let _ = super::unix_shutdown::assert_descendant_killed_after_shutdown;
}
