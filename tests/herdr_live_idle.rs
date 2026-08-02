//! Live herdr teardown check (opt-in via `MALVIN_LIVE_HERDR=1`).
//!
//! Verifies the stuck-animation fix by querying herdr (`herdr pane get`) for
//! `result.pane.agent_status` after a completed start/end cycle — including when
//! the pane was already stuck in `working` or sticky `unknown` beforehand.

#![cfg(unix)]

use serde_json::Value;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn live_enabled() -> bool {
    std::env::var_os("MALVIN_LIVE_HERDR").is_some()
}

fn require_herdr_env() -> (String, PathBuf) {
    assert_eq!(std::env::var("HERDR_ENV").ok().as_deref(), Some("1"));
    let pane = std::env::var("HERDR_PANE_ID").expect("HERDR_PANE_ID");
    let sock = PathBuf::from(std::env::var("HERDR_SOCKET_PATH").expect("HERDR_SOCKET_PATH"));
    (pane, sock)
}

fn pane_get(pane: &str) -> Value {
    let out = std::process::Command::new("herdr")
        .args(["pane", "get", pane])
        .output()
        .expect("herdr pane get");
    assert!(
        out.status.success(),
        "herdr pane get failed: status={:?} stderr={}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("herdr pane get json")
}

fn agent_status(v: &Value) -> Option<&str> {
    v["result"]["pane"]["agent_status"].as_str()
}

fn display_agent(v: &Value) -> Option<&str> {
    v["result"]["pane"]["display_agent"].as_str()
}

fn next_seq() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

fn send_raw(sock: &PathBuf, method: &str, mut params: Value) {
    params["seq"] = Value::from(next_seq());
    let req = serde_json::json!({
        "id": format!("live-test:{}", next_seq()),
        "method": method,
        "params": params,
    });
    let mut line = serde_json::to_string(&req).expect("serialize");
    line.push('\n');
    let mut stream = std::os::unix::net::UnixStream::connect(sock).expect("connect herdr sock");
    stream
        .set_read_timeout(Some(std::time::Duration::from_millis(500)))
        .ok();
    stream
        .set_write_timeout(Some(std::time::Duration::from_millis(500)))
        .ok();
    stream.write_all(line.as_bytes()).expect("write");
    let mut buf = [0_u8; 4096];
    let _ = stream.read(&mut buf);
}

/// Leave the pane in sticky `unknown` the way old `release_agent` teardown did.
fn force_sticky_unknown(pane: &str, sock: &PathBuf) {
    send_raw(
        sock,
        "pane.report_agent",
        serde_json::json!({
            "pane_id": pane,
            "source": "herdr:malvin",
            "agent": "malvin",
            "state": "working",
        }),
    );
    send_raw(
        sock,
        "pane.release_agent",
        serde_json::json!({
            "pane_id": pane,
            "source": "herdr:malvin",
            "agent": "malvin",
        }),
    );
    let v = pane_get(pane);
    assert_eq!(agent_status(&v), Some("unknown"), "setup sticky unknown: {v}");
}

#[test]
fn live_notify_run_end_leaves_pane_idle() {
    if !live_enabled() {
        return;
    }
    let (pane, sock) = require_herdr_env();

    let run_dir = std::env::temp_dir().join("malvin_live_herdr_end");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    malvin::herdr::notify_run_start(&run_dir);
    malvin::herdr::notify_run_end();

    let v = pane_get(&pane);
    assert_eq!(agent_status(&v), Some("idle"), "{v}");
    let _ = sock;
}

#[test]
fn live_stuck_unknown_then_do_cycle_leaves_idle() {
    if !live_enabled() {
        return;
    }
    let (pane, sock) = require_herdr_env();

    // Prior sticky animation state (old release_agent teardown).
    force_sticky_unknown(&pane, &sock);

    // New `--do`-style cycle: bind working, then end → idle + clear display.
    let run_dir = std::env::temp_dir().join("malvin_live_herdr_stuck_then_do");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    malvin::herdr::notify_run_start(&run_dir);
    let mid = pane_get(&pane);
    assert_eq!(agent_status(&mid), Some("working"), "during run: {mid}");

    malvin::herdr::notify_run_end();
    let end = pane_get(&pane);
    assert_eq!(agent_status(&end), Some("idle"), "after end: {end}");
    assert!(
        display_agent(&end).is_none(),
        "display_agent should be cleared after end: {end}"
    );
}

#[test]
fn live_stuck_working_then_end_leaves_idle() {
    if !live_enabled() {
        return;
    }
    let (pane, _sock) = require_herdr_env();

    let stuck_dir = std::env::temp_dir().join("malvin_live_herdr_stuck_working");
    std::fs::create_dir_all(&stuck_dir).expect("mkdir");
    malvin::herdr::notify_run_start(&stuck_dir);
    let stuck = pane_get(&pane);
    assert_eq!(agent_status(&stuck), Some("working"), "stuck setup: {stuck}");
    // Abandon without end in this process's session tracking by starting a new cycle
    // (notify_run_start replaces the bind). Then end must still leave idle.
    let run_dir = std::env::temp_dir().join("malvin_live_herdr_stuck_working_do");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    malvin::herdr::notify_run_start(&run_dir);
    malvin::herdr::notify_run_end();

    let end = pane_get(&pane);
    assert_eq!(agent_status(&end), Some("idle"), "after end: {end}");
    assert!(
        display_agent(&end).is_none(),
        "display_agent should be cleared after end: {end}"
    );
}
