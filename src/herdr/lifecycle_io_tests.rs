use crate::herdr::lifecycle_io_support::{
    agent_state_of, assert_bind_shape, assert_idle_then_clear_metadata, collect_until_deadline,
    collect_until_teardown_clear, herdr_test_env_lock, install_test_herdr_env, method_of,
    restore_test_herdr_env, spawn_request_collector, with_herdr_fixture,
};
use crate::herdr::{notify_reclaim, notify_run_end, notify_run_start, notify_working};
use crate::herdr::{reset_session_for_test, session_active_for_test, session_has_binding_for_test};
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::time::Duration;

#[test]
fn lifecycle_reports_session_working_idle_clear_over_socket() {
    let _g = herdr_test_env_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_session_for_test();
    with_herdr_fixture(|run_dir, rx| {
        notify_run_start(run_dir);
        assert!(session_active_for_test());
        notify_reclaim();
        notify_working();
        notify_run_end();
        assert!(!session_active_for_test());
        assert!(!session_has_binding_for_test());
        let reqs = collect_until_teardown_clear(rx);
        assert_bind_shape(&reqs);
        assert_idle_then_clear_metadata(&reqs);
    });
    reset_session_for_test();
}

#[test]
fn notify_working_pulses_working_without_clearing_authority() {
    let _g = herdr_test_env_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_session_for_test();
    with_herdr_fixture(|run_dir, rx| {
        notify_run_start(run_dir);
        let _ = collect_until_deadline(rx, Duration::from_millis(400));
        notify_working();
        let pulsed = collect_until_deadline(rx, Duration::from_millis(400));
        assert!(pulsed.iter().any(|v| {
            method_of(v) == "pane.report_agent" && agent_state_of(v) == Some("working")
        }));
        assert!(
            pulsed
                .iter()
                .all(|v| method_of(v) != "pane.clear_agent_authority")
        );
        notify_run_end();
        let _ = collect_until_teardown_clear(rx);
    });
    reset_session_for_test();
}

fn bind_test_socket_or_skip(sock: &Path) -> Option<UnixListener> {
    match UnixListener::bind(sock) {
        Ok(listener) => Some(listener),
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            eprintln!("skipping Unix-socket lifecycle test: bind denied: {error}");
            None
        }
        Err(error) => panic!("bind: {error}"),
    }
}

fn retry_teardown_after_socket_loss(sock: &Path, run_dir: &Path) -> bool {
    let Some(listener) = bind_test_socket_or_skip(sock) else {
        return false;
    };
    let rx = spawn_request_collector(listener);
    notify_run_start(run_dir);
    let _ = collect_until_deadline(&rx, Duration::from_millis(500));
    let _ = std::fs::remove_file(sock);
    notify_run_end();
    assert!(session_has_binding_for_test());
    assert!(!session_active_for_test());
    let Some(listener) = bind_test_socket_or_skip(sock) else {
        return false;
    };
    let rx2 = spawn_request_collector(listener);
    notify_run_end();
    assert_idle_then_clear_metadata(&collect_until_teardown_clear(&rx2));
    assert!(!session_has_binding_for_test());
    true
}

#[test]
fn failed_teardown_retains_binding_for_retry_then_clears() {
    let _g = herdr_test_env_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_session_for_test();
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("herdr.sock");
    let old = install_test_herdr_env(&sock);
    let run_dir = dir.path().join("retry_run");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    let _ = retry_teardown_after_socket_loss(&sock, &run_dir);
    restore_test_herdr_env(old);
    reset_session_for_test();
}
