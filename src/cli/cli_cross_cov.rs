//! Static entrypoint refs and focused behavioral smokes (`cli_smoke_cov.rs` holds the rest).

#[test]
fn kiss_wire_cli_entrypoint_refs() {
    let _ = crate::cli::entrypoint;
    let _ = crate::cli::run_code;
    let _ = crate::cli::run_do;
    let _ = crate::cli::run_ideas;
    let _ = crate::cli::run_tidy;
    let _ = crate::cli::run_plan;
}

#[test]
fn smoke_format_gate_failures_non_empty() {
    let pre = super::format_pre_check_gate_failure("malvin code", "kiss failed");
    assert!(pre.contains("kiss failed"));
    let ws = super::format_workspace_gate_failure("malvin tidy", "gate failed");
    assert!(ws.contains("gate failed"));
    let code = super::format_code_pre_check_failure("detail");
    assert!(code.contains("detail"));
}

#[test]
fn smoke_lookup_bin_on_path_finds_sh() {
    assert!(crate::lookup_bin_on_path("sh").is_some());
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
