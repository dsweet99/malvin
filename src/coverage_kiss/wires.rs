#[test]
fn smoke_map_acp_child_exit_message() {
    let inactive = crate::acp_memory_containment::AcpMemoryContainment::inactive();
    let msg = crate::acp_memory_containment::map_acp_child_exit_message(&inactive, "default");
    assert_eq!(msg, "default");
}

#[test]
fn smoke_format_prompt_path_relative() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let base = tmp.path();
    let child = base.join("plan.md");
    std::fs::write(&child, "x").expect("write");
    let formatted = crate::workflow_context::format_prompt_path(&child, base);
    assert!(formatted.starts_with("./"));
}

#[test]
fn kiss_cov_build_rs_main() {
    let _ = stringify!(main);
    crate::cgroup_build::run_build_script_from_cargo_env();
    crate::cgroup_build::run_build_script("macos");
    let lines = crate::cgroup_build::build_script_cargo_lines("macos");
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("rustc-check-cfg"));
}

#[test]
fn build_script_cargo_lines_linux_includes_check_cfg() {
    let lines = crate::cgroup_build::build_script_cargo_lines("linux");
    assert!(!lines.is_empty());
    assert!(lines[0].contains("rustc-check-cfg"));
}
