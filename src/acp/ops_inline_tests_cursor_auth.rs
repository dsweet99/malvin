use super::{clear_cursor_api_keys, restore_optional_env, write_path_executable};

use crate::acp::{MALVIN_TEST_NO_REAL_AGENT_ENV, cursor_cli_auth_established};

const EXIT0: &[u8] = b"#!/bin/sh\nexit 0\n";

fn assert_cursor_cli_auth_with_path_bin(bin_name: &str, body: &[u8]) {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().unwrap();
    let path_dir = tmp.path().join("bin");
    std::fs::create_dir(&path_dir).unwrap();
    write_path_executable(&path_dir.join(bin_name), body);
    unsafe {
        clear_cursor_api_keys();
    }
    let old_path = std::env::var_os("PATH");
    let old_no_real = std::env::var_os(MALVIN_TEST_NO_REAL_AGENT_ENV);

    unsafe {
        std::env::remove_var(MALVIN_TEST_NO_REAL_AGENT_ENV);
        std::env::set_var("PATH", &path_dir);
    }

    assert!(cursor_cli_auth_established());

    unsafe {
        restore_optional_env("PATH", old_path);
        restore_optional_env(MALVIN_TEST_NO_REAL_AGENT_ENV, old_no_real);
    }
}

#[test]
fn cursor_cli_auth_established_true_when_agent_auth_status_succeeds() {
    assert_cursor_cli_auth_with_path_bin("agent", EXIT0);
}

#[test]
fn cursor_cli_auth_established_true_when_cursor_agent_auth_status_succeeds() {
    assert_cursor_cli_auth_with_path_bin("cursor-agent", EXIT0);
}

#[test]
fn cursor_cli_auth_established_true_when_agent_whoami_succeeds() {
    assert_cursor_cli_auth_with_path_bin(
        "agent",
        b"#!/bin/sh\nif [ \"$1\" = auth ]; then exit 1; fi\nif [ \"$1\" = whoami ]; then exit 0; fi\nexit 1\n",
    );
}
