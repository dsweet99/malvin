#![allow(unsafe_code)]

use super::{reset_env_to_not_working, reset_to_not_working};
use crate::herdr::env::HerdrEnv;
use crate::herdr::lifecycle_io_support::{
    agent_state_of, herdr_test_env_lock, method_of, restore_test_herdr_env, with_herdr_fixture,
};
use std::path::PathBuf;
use std::time::Duration;

#[test]
fn reset_without_herdr_env_errors() {
    let _g = herdr_test_env_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let old = [
        std::env::var_os("MALVIN_TEST_HERDR_IO"),
        std::env::var_os("HERDR_ENV"),
        std::env::var_os("HERDR_SOCKET_PATH"),
        std::env::var_os("HERDR_PANE_ID"),
    ];
    unsafe {
        std::env::remove_var("MALVIN_TEST_HERDR_IO");
        std::env::remove_var("HERDR_ENV");
        std::env::remove_var("HERDR_SOCKET_PATH");
        std::env::remove_var("HERDR_PANE_ID");
    }
    let err = reset_to_not_working().expect_err("missing env");
    assert!(err.contains("herdr env not set"), "{err}");
    restore_test_herdr_env(old);
}

#[test]
fn reset_env_sends_idle_then_clear_metadata() {
    let _g = herdr_test_env_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    with_herdr_fixture(|_run_dir, rx| {
        let env = HerdrEnv {
            socket_path: PathBuf::from(std::env::var_os("HERDR_SOCKET_PATH").expect("sock")),
            pane_id: "test-pane".into(),
        };
        reset_env_to_not_working(&env).expect("reset");
        let first = rx.recv_timeout(Duration::from_secs(2)).expect("idle req");
        assert_eq!(method_of(&first), "pane.report_agent");
        assert_eq!(agent_state_of(&first), Some("idle"));
        let second = rx.recv_timeout(Duration::from_secs(2)).expect("clear req");
        assert_eq!(method_of(&second), "pane.report_metadata");
        assert_eq!(
            second["params"]["clear_display_agent"].as_bool(),
            Some(true)
        );
    });
}
