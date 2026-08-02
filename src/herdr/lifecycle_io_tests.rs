//! Integration-style herdr lifecycle tests against a local Unix socket.

#![allow(unsafe_code)]

use crate::herdr::{notify_run_end, notify_run_start, notify_working};
use crate::herdr::{reset_session_for_test, session_active_for_test};
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
    // SAFETY: caller holds `herdr_test_env_lock` for exclusive env mutation in these tests.
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
        std::env::set_var("MALVIN_TEST_HERDR_IO", "1");
        std::env::set_var("HERDR_ENV", "1");
        std::env::set_var("HERDR_SOCKET_PATH", sock);
        std::env::set_var("HERDR_PANE_ID", "test-pane");
    }
    old
}

fn restore_test_herdr_env(old: [Option<std::ffi::OsString>; 4]) {
    restore_env("MALVIN_TEST_HERDR_IO", old[0].clone());
    restore_env("HERDR_ENV", old[1].clone());
    restore_env("HERDR_SOCKET_PATH", old[2].clone());
    restore_env("HERDR_PANE_ID", old[3].clone());
}

fn spawn_method_collector(listener: UnixListener) -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || {
        for _ in 0..8 {
            let Ok((mut conn, _)) = listener.accept() else {
                break;
            };
            let mut line = String::new();
            if BufReader::new(&mut conn).read_line(&mut line).is_ok() {
                if let Ok(v) = serde_json::from_str::<Value>(line.trim()) {
                    if let Some(m) = v.get("method").and_then(Value::as_str) {
                        let _ = tx.send(m.to_string());
                    }
                }
            }
            let _ = conn.write_all(br#"{"result":{"type":"ok"}}"#);
        }
    });
    rx
}

fn collect_until_release(rx: &Receiver<String>) -> Vec<String> {
    let mut methods = Vec::new();
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while std::time::Instant::now() < deadline {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(m) => {
                methods.push(m);
                if methods.iter().any(|x| x == "pane.release_agent") {
                    break;
                }
            }
            Err(_) => {
                if methods.iter().any(|x| x == "pane.release_agent") {
                    break;
                }
            }
        }
    }
    methods
}

fn assert_lifecycle_methods(methods: &[String]) {
    for required in [
        "pane.report_agent_session",
        "pane.report_agent",
        "pane.release_agent",
        "pane.report_metadata",
    ] {
        assert!(
            methods.iter().any(|m| m == required),
            "missing {required}: {methods:?}"
        );
    }
}

fn run_lifecycle_against_socket(run_dir: &Path) -> Vec<String> {
    notify_run_start(run_dir);
    assert!(session_active_for_test());
    notify_working();
    notify_run_end();
    assert!(!session_active_for_test());
    Vec::new()
}

fn with_herdr_fixture(f: impl FnOnce(&Path, &Receiver<String>)) {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock: PathBuf = dir.path().join("herdr.sock");
    let rx = spawn_method_collector(UnixListener::bind(&sock).expect("bind"));
    let old = install_test_herdr_env(&sock);
    let run_dir = dir.path().join("20260802_test_run");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    f(&run_dir, &rx);
    restore_test_herdr_env(old);
}

#[test]
fn lifecycle_reports_session_working_idle_release_over_socket() {
    let _guard = herdr_test_env_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_session_for_test();
    with_herdr_fixture(|run_dir, rx| {
        let _ = run_lifecycle_against_socket(run_dir);
        assert_lifecycle_methods(&collect_until_release(rx));
    });
    reset_session_for_test();
}
