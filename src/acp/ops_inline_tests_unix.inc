fn write_path_executable(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    std::fs::write(path, b"#!/bin/sh\nexit 0\n").unwrap();
    let mut perms = std::fs::metadata(path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).unwrap();
    crate::test_utils::sync_test_executable(path);
}

#[test]
fn resolve_agent_bin_prefers_env_override() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().unwrap();
    let override_bin = tmp.path().join("custom-agent");
    write_path_executable(&override_bin);
    let path_dir = tmp.path().join("path-bin");
    std::fs::create_dir(&path_dir).unwrap();
    write_path_executable(&path_dir.join("agent"));
    let old_override = std::env::var_os("MALVIN_AGENT_ACP_BIN");
    let old_path = std::env::var_os("PATH");

    unsafe {
        std::env::set_var("MALVIN_AGENT_ACP_BIN", &override_bin);
        std::env::set_var("PATH", &path_dir);
    }

    assert_eq!(resolve_agent_bin().as_deref(), Some(override_bin.as_path()));

    unsafe {
        if let Some(value) = old_override {
            std::env::set_var("MALVIN_AGENT_ACP_BIN", value);
        } else {
            std::env::remove_var("MALVIN_AGENT_ACP_BIN");
        }
        if let Some(value) = old_path {
            std::env::set_var("PATH", value);
        } else {
            std::env::remove_var("PATH");
        }
    }
}

#[test]
fn resolve_agent_bin_prefers_agent_when_both_agent_and_cursor_on_path() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().unwrap();
    let path_dir = tmp.path().join("bin");
    std::fs::create_dir(&path_dir).unwrap();
    let agent_bin = path_dir.join("agent");
    let cursor_bin = path_dir.join("cursor-agent");
    write_path_executable(&agent_bin);
    write_path_executable(&cursor_bin);
    let old_override = std::env::var_os("MALVIN_AGENT_ACP_BIN");
    let old_path = std::env::var_os("PATH");

    unsafe {
        std::env::remove_var("MALVIN_AGENT_ACP_BIN");
        std::env::set_var("PATH", &path_dir);
    }

    assert_eq!(resolve_agent_bin().as_deref(), Some(agent_bin.as_path()));

    unsafe {
        if let Some(value) = old_override {
            std::env::set_var("MALVIN_AGENT_ACP_BIN", value);
        } else {
            std::env::remove_var("MALVIN_AGENT_ACP_BIN");
        }
        if let Some(value) = old_path {
            std::env::set_var("PATH", value);
        } else {
            std::env::remove_var("PATH");
        }
    }
}

#[test]
fn resolve_agent_bin_falls_back_to_cursor_agent_on_path() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().unwrap();
    let path_dir = tmp.path().join("bin");
    std::fs::create_dir(&path_dir).unwrap();
    let cursor_agent = path_dir.join("cursor-agent");
    write_path_executable(&cursor_agent);
    let old_override = std::env::var_os("MALVIN_AGENT_ACP_BIN");
    let old_path = std::env::var_os("PATH");

    unsafe {
        std::env::remove_var("MALVIN_AGENT_ACP_BIN");
        std::env::set_var("PATH", &path_dir);
    }

    assert_eq!(
        resolve_agent_bin().as_deref(),
        Some(cursor_agent.as_path())
    );

    unsafe {
        if let Some(value) = old_override {
            std::env::set_var("MALVIN_AGENT_ACP_BIN", value);
        } else {
            std::env::remove_var("MALVIN_AGENT_ACP_BIN");
        }
        if let Some(value) = old_path {
            std::env::set_var("PATH", value);
        } else {
            std::env::remove_var("PATH");
        }
    }
}
