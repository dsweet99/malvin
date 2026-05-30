//! Live end-to-end test: real `malvin` + real cursor-agent + real `store.db`.
//!
//! Run manually:
//! ```text
//! MALVIN_LIVE_DEFER_ENRICH=1 cargo nextest run defer_enrich_live -- --ignored
//! ```
//!
//! Requires: `agent` or `cursor-agent` on PATH, API key or logged-in CLI auth,
//! and network access. Uses the real `~/.cursor` session store (does not override
//! `HOME`).

#[cfg(unix)]
mod common;

#[cfg(unix)]
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::process::Command;

#[cfg(unix)]
use common::{
    command_output_live_agent, live_agent_prereqs_met, only_run_dir, LIVE_AGENT_CMD_TIMEOUT,
};

#[cfg(unix)]
const PROBE_FILE: &str = "defer_enrich_probe.rs";
#[cfg(unix)]
const PROBE_PATH_FRAGMENT: &str = "defer_enrich_probe.rs";
#[cfg(unix)]
const READ_FALLBACK: &str = "Read file ·";

#[cfg(unix)]
fn require_live_gate() {
    assert_eq!(
        std::env::var("MALVIN_LIVE_DEFER_ENRICH").ok().as_deref(),
        Some("1"),
        "set MALVIN_LIVE_DEFER_ENRICH=1 to run this test"
    );
    assert!(
        live_agent_prereqs_met(),
        "live defer-enrich e2e requires agent/cursor-agent on PATH and API key or logged-in auth"
    );
}

#[cfg(unix)]
fn seed_probe_workspace(workspace: &Path) {
    std::fs::create_dir_all(workspace.join(".malvin")).expect("mkdir .malvin");
    std::fs::write(workspace.join(".kissconfig"), "x").expect("kissconfig");
    std::fs::write(
        workspace.join(PROBE_FILE),
        "// defer_enrich_probe marker\npub fn defer_enrich_probe_marker() -> &'static str { \"DEFER_ENRICH_PROBE_7f3a\" }\n",
    )
    .expect("probe file");
}

#[cfg(unix)]
fn spawn_malvin_do_read_probe(workspace: &Path) -> std::process::Output {
    let prompt = format!(
        "Use the Read tool to read {PROBE_FILE} exactly once, then reply with only: OK"
    );
    command_output_live_agent(
        Command::new(env!("CARGO_BIN_EXE_malvin"))
            .current_dir(workspace)
            .env_remove("MALVIN_TEST_NO_REAL_AGENT")
            .env_remove("MALVIN_AGENT_ACP_BIN")
            .env("MALVIN_FORCE_STDOUT_TEE", "1")
            .env("MALVIN_DEFER_LOG_MAX_AGE_MS", "1500")
            .args(["do", &prompt]),
    )
    .unwrap_or_else(|e| panic!("malvin do timed out after {LIVE_AGENT_CMD_TIMEOUT:?}: {e}"))
}

#[cfg(unix)]
fn session_id_from_trace(trace_path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(trace_path).ok()?;
    for line in text.lines() {
        if !line.contains("sessionId") {
            continue;
        }
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        for pointer in [
            "/result/sessionId",
            "/params/sessionId",
            "/message/result/sessionId",
            "/message/params/sessionId",
        ] {
            if let Some(id) = v.pointer(pointer).and_then(serde_json::Value::as_str) {
                return Some(id.to_string());
            }
        }
    }
    None
}

#[cfg(unix)]
fn cursor_store_path(session_id: &str) -> PathBuf {
    malvin::user_home_dir()
        .join(".cursor")
        .join("acp-sessions")
        .join(session_id)
        .join("store.db")
}

#[cfg(unix)]
fn store_db_contains_path(store_db: &Path, needle: &str) -> bool {
    malvin::store_db_contains_substring(store_db, needle)
}

#[cfg(unix)]
fn combined_output(out: &std::process::Output) -> String {
    format!(
        "status={:?}\nstdout={}\nstderr={}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

#[cfg(unix)]
#[test]
#[ignore = "live cursor-agent e2e; MALVIN_LIVE_DEFER_ENRICH=1 cargo nextest run defer_enrich_live -- --ignored"]
fn defer_enrich_live_read_shows_path_in_stdout_log() {
    require_live_gate();
    let workspace = tempfile::tempdir().expect("tempdir");
    seed_probe_workspace(workspace.path());

    let out = spawn_malvin_do_read_probe(workspace.path());
    assert!(
        out.status.success(),
        "malvin do must succeed for live enrich e2e:\n{}",
        combined_output(&out)
    );

    let home = malvin::user_home_dir();
    let run_dir = only_run_dir(workspace.path(), &home);
    let stdout_log = run_dir.join("stdout.log");
    let stdout = std::fs::read_to_string(&stdout_log).unwrap_or_else(|e| {
        panic!(
            "stdout.log missing at {}: {e}",
            stdout_log.display()
        )
    });

    assert!(
        stdout.contains(PROBE_PATH_FRAGMENT),
        "stdout.log must show enriched read path {PROBE_PATH_FRAGMENT:?}; log={stdout:?}"
    );
    assert!(
        !stdout.contains(READ_FALLBACK),
        "stdout.log must not contain generic read fallback {READ_FALLBACK:?}; log={stdout:?}"
    );

    let trace_path = run_dir.join("trace.jsonl");
    let session_id = session_id_from_trace(&trace_path).unwrap_or_else(|| {
        panic!(
            "trace.jsonl must contain sessionId for store.db cross-check: {}",
            trace_path.display()
        )
    });
    let store_db = cursor_store_path(&session_id);
    assert!(
        store_db.is_file(),
        "Cursor store.db must exist for session {session_id}: {}",
        store_db.display()
    );
    assert!(
        store_db_contains_path(&store_db, PROBE_PATH_FRAGMENT),
        "store.db must contain read path {PROBE_PATH_FRAGMENT:?} for session {session_id}"
    );
}
