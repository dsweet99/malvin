use super::prelude::*;
use super::jsonrpc::*;
use super::shared_handshake::*;

#[test]
fn test_cursor_credentials_explicit_auth_token_overrides_process_env() {
    let _guard = crate::test_utils::test_env_lock();
    clear_cursor_env_for_test();
    unsafe {
        std::env::set_var("CURSOR_AUTH_TOKEN", "from-env");
    }
    let tmp = tempfile::tempdir().unwrap();
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: None,
        auth_token: Some("explicit-tok"),
        george_acp_lane: None,
        model: None,
        force: false,
    });
    assert_cursor_credentials_forwarding(&cmd, None, Some("explicit-tok"));
    clear_cursor_env_for_test();
}

/// Neither explicit nor `CURSOR_*` env: no credentials forwarded.
#[test]
fn test_cursor_credentials_absent_process_env_and_no_explicit() {
    let _guard = crate::test_utils::test_env_lock();
    clear_cursor_env_for_test();
    let tmp = tempfile::tempdir().unwrap();
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: None,
        auth_token: None,
        george_acp_lane: None,
        model: None,
        force: false,
    });
    assert_cursor_credentials_forwarding(&cmd, None, None);
}

/// Empty explicit key skips to `CURSOR_API_KEY` from the process when set.
#[test]
fn test_cursor_credentials_empty_explicit_key_falls_back_to_process_env() {
    let _guard = crate::test_utils::test_env_lock();
    clear_cursor_env_for_test();
    unsafe {
        std::env::set_var("CURSOR_API_KEY", "env-after-empty-explicit");
    }
    let tmp = tempfile::tempdir().unwrap();
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: Some(""),
        auth_token: None,
        george_acp_lane: None,
        model: None,
        force: false,
    });
    assert_cursor_credentials_forwarding(&cmd, Some("env-after-empty-explicit"), None);
    clear_cursor_env_for_test();
}

fn assert_empty_cursor_env_credential_ignored(env_var: &str) {
    let _guard = crate::test_utils::test_env_lock();
    clear_cursor_env_for_test();
    unsafe {
        std::env::set_var(env_var, "");
    }
    let tmp = tempfile::tempdir().unwrap();
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: None,
        auth_token: None,
        george_acp_lane: None,
        model: None,
        force: false,
    });
    assert_cursor_credentials_forwarding(&cmd, None, None);
    clear_cursor_env_for_test();
}

/// Empty `CURSOR_API_KEY` in the environment is ignored (treated as unset).
#[test]
fn test_cursor_credentials_process_env_empty_api_key_ignored() {
    assert_empty_cursor_env_credential_ignored("CURSOR_API_KEY");
}

/// Empty explicit token skips to `CURSOR_AUTH_TOKEN` from the process when set.
#[test]
fn test_cursor_credentials_empty_explicit_token_falls_back_to_process_env() {
    let _guard = crate::test_utils::test_env_lock();
    clear_cursor_env_for_test();
    unsafe {
        std::env::set_var("CURSOR_AUTH_TOKEN", "env-after-empty-explicit-tok");
    }
    let tmp = tempfile::tempdir().unwrap();
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: None,
        auth_token: Some(""),
        george_acp_lane: None,
        model: None,
        force: false,
    });
    assert_cursor_credentials_forwarding(&cmd, None, Some("env-after-empty-explicit-tok"));
    clear_cursor_env_for_test();
}

/// Empty `CURSOR_AUTH_TOKEN` in the environment is ignored.
#[test]
fn test_cursor_credentials_process_env_empty_auth_token_ignored() {
    assert_empty_cursor_env_credential_ignored("CURSOR_AUTH_TOKEN");
}

#[test]
fn test_cursor_credentials_token_only() {
    let _guard = crate::test_utils::test_env_lock();
    clear_cursor_env_for_test();
    let tmp = tempfile::tempdir().unwrap();
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: None,
        auth_token: Some("t-only"),
        george_acp_lane: None,
        model: None,
        force: false,
    });
    assert_cursor_credentials_forwarding(&cmd, None, Some("t-only"));
}

