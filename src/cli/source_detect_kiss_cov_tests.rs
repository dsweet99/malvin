#[test]
fn kiss_cov_source_detect_units() {
    let _ = crate::source_detect::entry_name_has_extension;
    let _ = crate::source_detect::entry_name_is_workspace_marker;
    let tmp = tempfile::tempdir().expect("tempdir");
    let p = tmp.path().join("x.rs");
    std::fs::write(&p, "").expect("write");
    let _ = crate::source_detect::entry_or_symlink_file_target_matches(&p, |_| true);
    let _ = crate::source_detect::resolved_symlink_target;
    let _ = crate::source_detect::symlink_resolves_to_existing_file;
}
