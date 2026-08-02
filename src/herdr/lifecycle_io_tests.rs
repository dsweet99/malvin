//! Integration-style herdr lifecycle tests against a local Unix socket.

#![allow(unsafe_code)]

use crate::herdr::{notify_reclaim, notify_run_end, notify_run_start, notify_working};
use crate::herdr::{
    reset_session_for_test, session_active_for_test, session_has_binding_for_test,
};
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

fn herdr_test_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn restore_env(key: &str, old: Option<std::ffi::OsString>) {
    // SAFETY: exclusive under `herdr_test_env_lock`.
    unsafe {
        match old {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }
}

fn install_test_herdr_env(sock: &Path) -> [Option<std::ffi::OsString>; 4] {
    let old = [
        std::env::var_os("MALVIN_TEST_HERDR_IO"),
        std::env::var_os("HERDR_ENV"),
        std::env::var_os("HERDR_SOCKET_PATH"),
        std::env::var_os("HERDR_PANE_ID"),
    ];
    // SAFETY: exclusive under `herdr_test_env_lock`.
    unsafe {
        std::env::set_var("HERDR_SOCKET_PATH", sock);
        std::env::set_var("HERDR_PANE_ID", "test-pane");
        std::env::set_var("HERDR_ENV", "1");
        std::env::set_var("MALVIN_TEST_HERDR_IO", "1");
    }
    old
}

fn restore_test_herdr_env(old: [Option<std::ffi::OsString>; 4]) {
    restore_env("MALVIN_TEST_HERDR_IO", old[0].clone());
    restore_env("HERDR_ENV", old[1].clone());
    restore_env("HERDR_SOCKET_PATH", old[2].clone());
    restore_env("HERDR_PANE_ID", old[3].clone());
}

fn spawn_request_collector(listener: UnixListener) -> Receiver<Value> {
    let (tx, rx) = mpsc::channel::<Value>();
    thread::spawn(move || {
        for _ in 0..32 {
            let Ok((mut conn, _)) = listener.accept() else {
                break;
            };
            let mut line = String::new();
            if BufReader::new(&mut conn).read_line(&mut line).is_ok() {
                if let Ok(v) = serde_json::from_str::<Value>(line.trim()) {
                    let _ = tx.send(v);
                }
            }
            let _ = conn.write_all(br#"{"result":{"type":"ok"}}"#);
        }
    });
    rx
}

fn method_of(v: &Value) -> &str {
    v.get("method").and_then(Value::as_str).unwrap_or("")
}

fn agent_state_of(v: &Value) -> Option<&str> {
    v.get("params")
        .and_then(|p| p.get("state"))
        .and_then(Value::as_str)
}

fn is_clear_metadata_teardown(v: &Value) -> bool {
    method_of(v) == "pane.report_metadata"
        && v.get("params")
            .and_then(|p| p.get("clear_display_agent"))
            .and_then(Value::as_bool)
            == Some(true)
}

fn collect_until_teardown_clear(rx: &Receiver<Value>) -> Vec<Value> {
    let mut requests = Vec::new();
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while std::time::Instant::now() < deadline {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(v) => {
                let done = is_clear_metadata_teardown(&v);
                requests.push(v);
                if done {
                    break;
                }
            }
            Err(_) => {
                if requests.iter().any(is_clear_metadata_teardown) {
                    break;
                }
            }
        }
    }
    requests
}

fn collect_until_deadline(rx: &Receiver<Value>, budget: Duration) -> Vec<Value> {
    let mut out = Vec::new();
    let deadline = std::time::Instant::now() + budget;
    while std::time::Instant::now() < deadline {
        if let Ok(v) = rx.recv_timeout(Duration::from_millis(50)) {
            out.push(v);
        }
    }
    out
}

fn assert_idle_then_clear_metadata(requests: &[Value]) {
    let idle_at = requests
        .iter()
        .position(|v| method_of(v) == "pane.report_agent" && agent_state_of(v) == Some("idle"))
        .expect("idle");
    let clear_at = requests
        .iter()
        .position(is_clear_metadata_teardown)
        .expect("clear metadata");
    assert!(idle_at < clear_at, "idle before clear-metadata: {requests:?}");
    assert!(
        requests[idle_at + 1..].iter().all(|v| {
            !(method_of(v) == "pane.report_agent" && agent_state_of(v) == Some("working"))
        }),
        "no working after idle: {requests:?}"
    );
    assert!(
        requests.iter().all(|v| method_of(v) != "pane.release_agent"),
        "must not release_agent: {requests:?}"
    );
}

fn with_herdr_fixture(f: impl FnOnce(&Path, &Receiver<Value>)) {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("herdr.sock");
    let rx = spawn_request_collector(UnixListener::bind(&sock).expect("bind"));
    let old = install_test_herdr_env(&sock);
    let run_dir = dir.path().join("20260802_test_run");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    f(&run_dir, &rx);
    restore_test_herdr_env(old);
}

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
        for m in [
            "pane.clear_agent_authority",
            "pane.report_agent_session",
            "pane.report_agent",
            "pane.report_metadata",
        ] {
            assert!(reqs.iter().any(|v| method_of(v) == m), "missing {m}: {reqs:?}");
        }
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
        assert!(pulsed.iter().all(|v| method_of(v) != "pane.clear_agent_authority"));
        notify_run_end();
        let _ = collect_until_teardown_clear(rx);
    });
    reset_session_for_test();
}

fn retry_teardown_after_socket_loss(sock: &Path, run_dir: &Path) {
    let rx = spawn_request_collector(UnixListener::bind(sock).expect("bind"));
    notify_run_start(run_dir);
    let _ = collect_until_deadline(&rx, Duration::from_millis(500));
    let _ = std::fs::remove_file(sock);
    notify_run_end();
    assert!(session_has_binding_for_test());
    assert!(!session_active_for_test());
    let rx2 = spawn_request_collector(UnixListener::bind(sock).expect("rebind"));
    notify_run_end();
    assert_idle_then_clear_metadata(&collect_until_teardown_clear(&rx2));
    assert!(!session_has_binding_for_test());
}

#[test]
fn failed_teardown_retains_binding_for_retry_then_clears() {
    let _g = herdr_test_env_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_session_for_test();
    let dir = tempfile::tempdir().expect("tempdir");
    let sock: PathBuf = dir.path().join("herdr.sock");
    let old = install_test_herdr_env(&sock);
    let run_dir = dir.path().join("retry_run");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    retry_teardown_after_socket_loss(&sock, &run_dir);
    restore_test_herdr_env(old);
    reset_session_for_test();
}
