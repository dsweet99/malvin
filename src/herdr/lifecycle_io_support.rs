#![allow(unsafe_code)]

use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

pub fn herdr_test_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn restore_env(key: &str, old: Option<std::ffi::OsString>) {
    unsafe {
        match old {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }
}

pub fn install_test_herdr_env(sock: &Path) -> [Option<std::ffi::OsString>; 4] {
    let old = [
        std::env::var_os("MALVIN_TEST_HERDR_IO"),
        std::env::var_os("HERDR_ENV"),
        std::env::var_os("HERDR_SOCKET_PATH"),
        std::env::var_os("HERDR_PANE_ID"),
    ];
    unsafe {
        std::env::set_var("HERDR_SOCKET_PATH", sock);
        std::env::set_var("HERDR_PANE_ID", "test-pane");
        std::env::set_var("HERDR_ENV", "1");
        std::env::set_var("MALVIN_TEST_HERDR_IO", "1");
    }
    old
}

pub fn restore_test_herdr_env(old: [Option<std::ffi::OsString>; 4]) {
    restore_env("MALVIN_TEST_HERDR_IO", old[0].clone());
    restore_env("HERDR_ENV", old[1].clone());
    restore_env("HERDR_SOCKET_PATH", old[2].clone());
    restore_env("HERDR_PANE_ID", old[3].clone());
}

pub fn spawn_request_collector(listener: UnixListener) -> Receiver<Value> {
    let (tx, rx) = mpsc::channel::<Value>();
    thread::spawn(move || {
        for _ in 0..32 {
            let Ok((mut conn, _)) = listener.accept() else {
                break;
            };
            let mut line = String::new();
            if BufReader::new(&mut conn).read_line(&mut line).is_ok()
                && let Ok(v) = serde_json::from_str::<Value>(line.trim())
            {
                let _ = tx.send(v);
            }
            let _ = conn.write_all(br#"{"result":{"type":"ok"}}"#);
        }
    });
    rx
}

pub fn method_of(v: &Value) -> &str {
    v.get("method").and_then(Value::as_str).unwrap_or("")
}

pub fn agent_state_of(v: &Value) -> Option<&str> {
    v.get("params")
        .and_then(|p| p.get("state"))
        .and_then(Value::as_str)
}

pub fn is_clear_metadata_teardown(v: &Value) -> bool {
    method_of(v) == "pane.report_metadata"
        && v.get("params")
            .and_then(|p| p.get("clear_display_agent"))
            .and_then(Value::as_bool)
            == Some(true)
}

pub fn collect_until_teardown_clear(rx: &Receiver<Value>) -> Vec<Value> {
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

pub fn collect_until_deadline(rx: &Receiver<Value>, budget: Duration) -> Vec<Value> {
    let mut out = Vec::new();
    let deadline = std::time::Instant::now() + budget;
    while std::time::Instant::now() < deadline {
        if let Ok(v) = rx.recv_timeout(Duration::from_millis(50)) {
            out.push(v);
        }
    }
    out
}

pub fn assert_idle_then_clear_metadata(requests: &[Value]) {
    let idle_at = requests
        .iter()
        .position(|v| method_of(v) == "pane.report_agent" && agent_state_of(v) == Some("idle"))
        .expect("idle");
    let clear_at = requests
        .iter()
        .position(is_clear_metadata_teardown)
        .expect("clear metadata");
    assert!(
        idle_at < clear_at,
        "idle before clear-metadata: {requests:?}"
    );
    assert!(
        requests[idle_at + 1..].iter().all(|v| {
            !(method_of(v) == "pane.report_agent" && agent_state_of(v) == Some("working"))
        }),
        "no working after idle: {requests:?}"
    );
    assert!(
        requests
            .iter()
            .all(|v| method_of(v) != "pane.release_agent"),
        "must not release_agent: {requests:?}"
    );
}

pub fn with_herdr_fixture(f: impl FnOnce(&Path, &Receiver<Value>)) {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("herdr.sock");
    let listener = match UnixListener::bind(&sock) {
        Ok(listener) => listener,
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            eprintln!("skipping Unix-socket lifecycle test: bind denied: {error}");
            return;
        }
        Err(error) => panic!("bind: {error}"),
    };
    let rx = spawn_request_collector(listener);
    let old = install_test_herdr_env(&sock);
    let run_dir = dir.path().join("20260802_test_run");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    f(&run_dir, &rx);
    restore_test_herdr_env(old);
}

pub fn assert_bind_shape(reqs: &[Value]) {
    for m in [
        "pane.clear_agent_authority",
        "pane.report_agent_session",
        "pane.report_agent",
        "pane.report_metadata",
        "agent.rename",
    ] {
        assert!(
            reqs.iter().any(|v| method_of(v) == m),
            "missing {m}: {reqs:?}"
        );
    }
    let rename = reqs
        .iter()
        .find(|v| method_of(v) == "agent.rename")
        .expect("rename");
    assert_eq!(rename["params"]["name"], "mrun", "rename name: {rename}");
    assert_title_not_run_basename(reqs);
}

pub fn assert_title_not_run_basename(reqs: &[Value]) {
    for v in reqs {
        if method_of(v) != "pane.report_metadata" || is_clear_metadata_teardown(v) {
            continue;
        }
        if let Some(t) = v
            .get("params")
            .and_then(|p| p.get("title"))
            .and_then(Value::as_str)
        {
            assert_ne!(
                t, "20260802_test_run",
                "title must not be run-dir basename: {v}"
            );
        }
    }
}
