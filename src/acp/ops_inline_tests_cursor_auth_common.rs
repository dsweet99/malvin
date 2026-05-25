use crate::acp::{
    AgentClient, AgentIoOptions, MALVIN_TEST_NO_REAL_AGENT_ENV, cursor_cli_auth_established,
};

use super::clear_cursor_api_keys;

#[test]
fn cursor_cli_auth_established_true_when_api_key_in_env() {
    let _guard = crate::test_utils::test_env_lock();
    unsafe {
        clear_cursor_api_keys();
        std::env::set_var("CURSOR_API_KEY", "test-key");
    }
    assert!(cursor_cli_auth_established());
    unsafe {
        std::env::remove_var("CURSOR_API_KEY");
    }
}

#[test]
fn cursor_cli_auth_established_false_when_no_key_and_real_agent_disabled() {
    let _guard = crate::test_utils::test_env_lock();
    unsafe {
        clear_cursor_api_keys();
    }
    let old_no_real_agent = std::env::var_os(MALVIN_TEST_NO_REAL_AGENT_ENV);

    unsafe {
        std::env::set_var(MALVIN_TEST_NO_REAL_AGENT_ENV, "1");
    }

    assert!(!cursor_cli_auth_established());

    unsafe {
        super::restore_optional_env(MALVIN_TEST_NO_REAL_AGENT_ENV, old_no_real_agent);
    }
}

#[test]
fn ensure_authenticated_err_when_no_credentials_and_probes_disabled() {
    let _guard = crate::test_utils::test_env_lock();
    unsafe {
        clear_cursor_api_keys();
        std::env::set_var(MALVIN_TEST_NO_REAL_AGENT_ENV, "1");
    }
    let client = AgentClient::new(
        "auth-test".to_string(),
        AgentIoOptions {
            force: false,
            no_sandbox: true,
            no_tee: false,
            raw_output: false,
            show_thoughts_on_stdout: true,
            emit_stdout_markdown: false,
            log_full_outgoing_prompts: false,
        },
    );
    let err = client.ensure_authenticated().unwrap_err();
    assert!(
        err.0.contains("not authenticated"),
        "unexpected error: {}",
        err.0
    );
    unsafe {
        std::env::remove_var(MALVIN_TEST_NO_REAL_AGENT_ENV);
    }
}
