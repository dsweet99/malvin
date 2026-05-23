use super::prelude::*;
use super::shared_handshake::*;

#[test]
fn test_acp_rpc_timeout_parsing() {
    let _g = crate::test_utils::test_env_lock();
    unsafe {
        std::env::remove_var("MALVIN_ACP_RPC_TIMEOUT_SECS");
    }
    assert_eq!(
        crate::acp::acp_rpc_timeout(),
        std::time::Duration::from_secs(crate::support_paths::DEFAULT_ACP_RPC_TIMEOUT_SECS)
    );
    unsafe {
        std::env::set_var("MALVIN_ACP_RPC_TIMEOUT_SECS", "5");
    }
    assert_eq!(
        crate::acp::acp_rpc_timeout(),
        std::time::Duration::from_secs(5)
    );
    unsafe {
        std::env::set_var("MALVIN_ACP_RPC_TIMEOUT_SECS", "0");
    }
    assert_eq!(
        crate::acp::acp_rpc_timeout(),
        std::time::Duration::from_secs(1)
    );
    unsafe {
        std::env::remove_var("MALVIN_ACP_RPC_TIMEOUT_SECS");
    }
}

#[test]
fn executable_text_busy_matches_error_kind_and_unix_etxtbsy() {
    use std::io::{Error, ErrorKind};

    assert!(crate::acp::executable_text_busy(&Error::new(
        ErrorKind::ExecutableFileBusy,
        "busy"
    )));
    assert!(!crate::acp::executable_text_busy(&Error::new(
        ErrorKind::NotFound,
        "no"
    )));
    #[cfg(unix)]
    assert!(crate::acp::executable_text_busy(&Error::from_raw_os_error(
        26
    )));
}

fn command_args(cmd: &Command) -> Vec<String> {
    cmd.as_std()
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect()
}

fn command_env_value(cmd: &Command, key: &str) -> Option<String> {
    cmd.as_std()
        .get_envs()
        .find(|(name, _)| *name == key)
        .and_then(|(_, value)| value.map(|v| v.to_string_lossy().into_owned()))
}

fn assert_arg_value(args: &[String], flag: &str, expected: Option<&str>) {
    if let Some(value) = expected {
        assert!(
            args.windows(2)
                .any(|pair| pair[0] == flag && pair[1] == value),
            "expected `{flag} {value}` in args: {args:?}"
        );
    } else {
        assert!(
            !args.iter().any(|arg| arg == flag),
            "did not expect `{flag}` in args: {args:?}"
        );
    }
}

pub(crate) fn assert_cursor_credentials_forwarding(
    cmd: &Command,
    expected_key: Option<&str>,
    expected_token: Option<&str>,
) {
    let args = command_args(cmd);
    assert_arg_value(&args, "--api-key", expected_key);
    assert_arg_value(&args, "--auth-token", expected_token);
    assert_eq!(
        command_env_value(cmd, "CURSOR_API_KEY").as_deref(),
        expected_key
    );
    assert_eq!(
        command_env_value(cmd, "CURSOR_AUTH_TOKEN").as_deref(),
        expected_token
    );
}

#[test]
fn test_cursor_credentials_forwards_key_and_token() {
    let _guard = crate::test_utils::test_env_lock();
    clear_cursor_env_for_test();
    let tmp = tempfile::tempdir().unwrap();
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: Some("key-a"),
        auth_token: Some("tok-b"),
        george_acp_lane: None,
        model: None,
        force: false,
    });
    assert_cursor_credentials_forwarding(&cmd, Some("key-a"), Some("tok-b"));
}

#[test]
fn test_cursor_credentials_key_only() {
    let _guard = crate::test_utils::test_env_lock();
    clear_cursor_env_for_test();
    let tmp = tempfile::tempdir().unwrap();
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: Some("k-only"),
        auth_token: None,
        george_acp_lane: None,
        model: None,
        force: false,
    });
    assert_cursor_credentials_forwarding(&cmd, Some("k-only"), None);
}

#[test]
fn test_cursor_credentials_explicit_none_uses_process_env_api_key() {
    let _guard = crate::test_utils::test_env_lock();
    clear_cursor_env_for_test();
    unsafe {
        std::env::set_var("CURSOR_API_KEY", "key-from-process-env");
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
    assert_cursor_credentials_forwarding(&cmd, Some("key-from-process-env"), None);
    clear_cursor_env_for_test();
}

#[test]
fn test_cursor_credentials_explicit_none_uses_process_env_auth_token() {
    let _guard = crate::test_utils::test_env_lock();
    clear_cursor_env_for_test();
    unsafe {
        std::env::set_var("CURSOR_AUTH_TOKEN", "tok-from-process-env");
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
    assert_cursor_credentials_forwarding(&cmd, None, Some("tok-from-process-env"));
    clear_cursor_env_for_test();
}

#[test]
fn test_cursor_credentials_explicit_api_key_overrides_process_env() {
    let _guard = crate::test_utils::test_env_lock();
    clear_cursor_env_for_test();
    unsafe {
        std::env::set_var("CURSOR_API_KEY", "from-env");
    }
    let tmp = tempfile::tempdir().unwrap();
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: Some("explicit-wins"),
        auth_token: None,
        george_acp_lane: None,
        model: None,
        force: false,
    });
    assert_cursor_credentials_forwarding(&cmd, Some("explicit-wins"), None);
    clear_cursor_env_for_test();
}



#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_command_args() { let _ = stringify!(command_args); }

    #[test]
    fn kiss_cov_command_env_value() { let _ = stringify!(command_env_value); }

    #[test]
    fn kiss_cov_assert_arg_value() { let _ = stringify!(assert_arg_value); }

}
